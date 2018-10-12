use post_graph::Graph;

use errors::IOError;
use posts::PostTypes;
use pulldown_cmark::{html, Parser};
use quick_error::ResultExt;
use std;
use std::fs::{create_dir_all, File};
use std::io::prelude::Write;
use std::path::{Path, PathBuf};
use symlink::symlink_file;
use tera;
use tera::{to_value, Context, Tera};

#[derive(Debug)]
pub struct PostHtml {
    filename: String,
    html: String,
}

pub fn get_templates(templateglob: &str) -> Tera {
    let mut tera = compile_templates!(templateglob);
    // and we can add more things to our instance if we want to
    //tera.autoescape_on(vec![".jinja2.html"]);
    tera.register_filter("parsemd", |value, _| {
        let s = try_get_value!("parsemd", "value", String, value);
        let parser = Parser::new(&s);

        let mut html_buf = String::new();
        html::push_html(&mut html_buf, parser);
        Ok(to_value(html_buf).unwrap())
    });
    tera
}

fn gen_post(tera: &Tera, post: &PostTypes, graph: &Graph) -> Result<PostHtml, tera::Error> {
    let mut ctx = Context::new();
    let children = graph.get_children(post);
    let html = match post {
        PostTypes::Post(p) => {
            ctx.insert("name", &p.name);
            ctx.insert("body", &p.body);
            ctx.insert("children", &children);
            ctx.insert("year", &p.year);
            ctx.insert("image", &p.image);
            ctx.insert("dlurl", &p.dl_url);
            tera.render("post.jinja2", &ctx)
        }
        PostTypes::Category(ref c) => {
            ctx.insert("name", &c.name);
            ctx.insert("body", &c.body);
            ctx.insert("children", &children);
            ctx.insert("year", "1998");
            ctx.insert("image", "");
            ctx.insert("dlurl", "");
            tera.render("category.jinja2", &ctx)
        }
    };
    match html {
        Ok(s) => Ok(PostHtml {
            filename: format!("{}.html", post.name().to_string()),
            html: s,
        }),
        Err(e) => Err(e),
    }
}
pub fn gen_posts_html(
    tera: &Tera,
    posts: &[PostTypes],
    graph: &Graph,
) -> Result<Vec<PostHtml>, Vec<tera::Error>> {
    let (posts, errors): (Vec<_>, Vec<_>) = posts
        .into_iter()
        .map(|p| gen_post(tera, p, graph))
        .partition(Result::is_ok);
    if errors.is_empty() {
        Ok(posts.into_iter().map(Result::unwrap).collect())
    } else {
        Err(errors.into_iter().map(Result::unwrap_err).collect())
    }
}

pub fn create_posts(srcdir: &Path, posts: &[PostHtml]) -> Result<(), IOError> {
    // write post to sourcedir
    posts
        .iter()
        .map(|post| {
            let filepath = srcdir.join(post.filename.clone());
            let mut output = File::create(&filepath).context(&filepath)?;
            output
                .write_all((&post.html).as_bytes())
                .context(&filepath)?;
            trace!("Created post -- file {:?}", filepath);
            Ok(())
        }).collect()
}
pub fn create_symlinks(wwwdir: &Path, srcdir: &Path, graph: &Graph) -> Result<(), IOError> {
    // generate symlinks and directories to each post as necessary
    let paths = graph.find_all_paths();
    debug!("Creating {} symlinks", paths.len());
    for (target, path) in paths {
        let targetname = format!("{}.html", target);
        let targetdir: PathBuf = wwwdir.join(path.iter().collect::<PathBuf>());
        let targetfile = targetdir.join(&targetname);
        let srcfile = path
            .iter()
            .map(|_| "../")
            .collect::<PathBuf>()
            .join("../")
            .join(&srcdir)
            .join(&targetname);

        // then create the symlink: targetpath -> sourcefile
        trace!(
            "Creating symlink -- file: {:?} -> original:{:?}",
            targetfile,
            srcfile
        );
        //let srcfile = srcdir.join(target);

        // now we need to create the relevant directories (targetdir)
        // ignore any pre-existing directories; overlapping creation is fine.
        if let Err(e) = create_dir_all(&targetdir) {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(e).context(&targetdir)?;
            }
        }
        symlink_file(&srcfile, &targetfile).context(&srcfile)?;
    }
    Ok(())
}
