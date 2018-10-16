use post_graph::Graph;

use errors::IOError;
use posts::PostTypes;
use pulldown_cmark::{html, Parser};
use quick_error::ResultExt;
use std;
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::prelude::Write;
use std::iter;
use std::path::{Path, PathBuf};
use symlink::symlink_file;
use tera;
use tera::{from_value, to_value, Context, Tera};
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

    tera.register_function(
        "joindir",
        Box::new(move |args| -> tera::Result<tera::Value> {
            match (args.get("d"), args.get("n")) {
                (Some(dirname), Some(name)) => match (
                    from_value::<String>(dirname.clone()),
                    from_value::<String>(name.clone()),
                ) {
                    (Ok(ref d), Ok(ref n)) if d == "" => Ok(to_value(n).unwrap()),
                    (Ok(ref d), Ok(ref n)) => Ok(to_value(format!("{}/{}", d, n)).unwrap()),
                    _ => Err("oops".into()),
                },
                _ => Err("oops".into()),
            }
        }),
    );
    tera
}

fn gen_post(
    tera: &Tera,
    post: &PostTypes,
    graph: &Graph,
    basepath: &str,
) -> Result<PostHtml, tera::Error> {
    let mut ctx = Context::new();

    let html = match post {
        PostTypes::Post(p) => {
            ctx.insert("post", &p);
            ctx.insert("children", &p.children);
            ctx.insert("basepath", &basepath);
            tera.render("post.jinja2", &ctx)
        }
        PostTypes::Category(c) => {
            //ctx.insert("category", c);
            ctx.insert("cat", c);
            ctx.insert(
                "childcats",
                &graph.get_child_cats(&graph.getidx(&post.name())),
            );
            ctx.insert("basepath", &basepath);
            ctx.insert(
                "childposts",
                &graph.get_child_posts(&graph.getidx(&post.name())),
            );
            tera.render("category.jinja2", &ctx)
        }
    };
    match html {
        Ok(s) => Ok(PostHtml {
            filename: format!("{}.html", post.name()),
            html: s,
        }),
        Err(e) => Err(e),
    }
}
fn gen_root(tera: &Tera, graph: &Graph, basepath: &str) -> Result<PostHtml, tera::Error> {
    let mut ctx = Context::new();
    ctx.insert("childcats", &graph.get_child_cats(&graph.root));
    ctx.insert("childposts", &graph.get_child_posts(&graph.root));
    ctx.insert("basepath", &basepath);
    let html = tera.render("index.jinja2", &ctx);
    match html {
        Ok(s) => Ok(PostHtml {
            filename: "index.html".to_string(),
            html: s,
        }),
        Err(e) => Err(e),
    }
}
pub fn gen_posts_html(
    tera: &Tera,
    posts: &[PostTypes],
    graph: &Graph,
    basepath: &str,
) -> Result<Vec<PostHtml>, Vec<tera::Error>> {
    let (posts, errors): (Vec<_>, Vec<_>) = posts
        .iter()
        .map(|p| gen_post(tera, p, graph, basepath))
        .chain(iter::once(gen_root(tera, graph, basepath))) // inject the index node
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
            let filepath = srcdir.join(&post.filename);
            trace!("Creating post -- file {:?}", filepath);

            // create directories for the post as necessary
            if let Some(dirs) = filepath.parent() {
                if let Err(e) = create_dir_all(dirs) {
                    if e.kind() != std::io::ErrorKind::AlreadyExists {
                        return Err(e).context(dirs)?;
                    }
                }
            }

            let mut output = File::create(&filepath).context(&filepath)?;

            output
                .write_all((&post.html).as_bytes())
                .context(&filepath)?;
            trace!("Created post -- file {:?}", filepath);
            Ok(())
        }).collect()
}
pub fn create_symlinks(wwwdir: &Path, srcdir: &Path, posts: &[PostHtml]) -> Result<(), IOError> {
    posts
        .iter()
        .map(|post| {
            let symfile = wwwdir.join(&post.filename);
            let srcfile = Path::new("../").join(srcdir).join(&post.filename);
            symlink_file(&srcfile, &symfile).context(&srcfile)?;
            Ok(())
        }).collect()
    //Ok(())
}
// pub fn create_symlinks(wwwdir: &Path, srcdir: &Path, graph: &Graph) -> Result<(), IOError> {
//     // generate symlinks and directories to each post as necessary
//     //
//     let paths = graph.find_all_paths();
//     debug!("Creating {} symlinks", paths.len());
//     for (target, path, m_dupix) in paths {
//         info!("{}, {:?}, {:?}",target, path, m_dupix);
//         let targetname = format!("{}.html", target);
//         let targetdir: PathBuf = wwwdir.join(path.iter().collect::<PathBuf>());
//         let targetfile = targetdir.join(&targetname);
//         let srcfile = path
//             .iter()
//             // if this is a cycle (in which case, m_dupix exists), then move back up to the previous symlink-copy.
//             // Else, we need to point to real html file, by going all the way back up to the root (www/).
//             .skip(m_dupix.unwrap_or(0))
//             // we need to do this with relative paths, because the whole directory will likely be moved later.
//             .map(|_| "../")
//             .collect::<PathBuf>()
//             .join("../")
//             .join(&srcdir)
//             .join(&targetname);

//         // then create the symlink: targetpath -> sourcefile
//         trace!(
//             "Creating symlink -- file: {:?} -> original:{:?}",
//             targetfile,
//             srcfile
//         );
//         //let srcfile = srcdir.join(target);

//         // now we need to create the relevant directories (targetdir)
//         // ignore any pre-existing directories; overlapping creation is fine.
//         if let Err(e) = create_dir_all(&targetdir) {
//             if e.kind() != std::io::ErrorKind::AlreadyExists {
//                 return Err(e).context(&targetdir)?;
//             }
//         }
//         symlink_file(&srcfile, &targetfile).context(&srcfile)?;
//     }
//     Ok(())
// }
