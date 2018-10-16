extern crate petgraph;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate serde_derive;
extern crate pulldown_cmark;
#[macro_use]
extern crate tera;
extern crate indicatif;
extern crate symlink;
extern crate toml;
#[macro_use]
extern crate log;
extern crate clap;
extern crate dialoguer;
extern crate iron;
extern crate mount;
extern crate simplelog;
extern crate staticfile;

mod errors;
mod gen_html;
mod post_graph;
mod posts;

use dialoguer::Confirmation;
use gen_html::{create_posts, create_symlinks, gen_posts_html, get_templates};
use post_graph::Graph;
use posts::PostTypes;
use simplelog::{Config, LevelFilter, TermLogger};
use std::fmt::{Debug, Display};
use std::fs::{copy, create_dir, create_dir_all, read_dir, remove_dir_all};
use std::path::{Path, PathBuf};

use clap::{App, Arg, ArgMatches};

fn argparse<'a>() -> ArgMatches<'a> {
    App::new("redsystem")
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            Arg::with_name("v")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Use verbose output. Repeat to increase verbosity, up to 3 times."),
        ).arg(
            Arg::with_name("outdir")
                .long("outdir")
                .short("o")
                .help("Directory to write generated files to")
                .takes_value(true)
                .default_value("./www"),
        ).arg(
            Arg::with_name("delete_outdir")
                .long("force-delete")
                .short("f")
                .help("delete outdir without prompting"),
        ).arg(
            Arg::with_name("templatedir")
                .long("templates")
                .short("t")
                .help("Directory to fetch html templates and css from")
                .takes_value(true)
                .default_value("./templates"),
        ).arg(
            Arg::with_name("postdir")
                .long("posts")
                .short("p")
                .help("Directory to fetch content files from")
                .takes_value(true)
                .default_value("./posts"),
        ).arg(
            Arg::with_name("basepath")
                .long("base-path")
                .short("b")
                .help("Base path to set in the html, if you're not hosting from root.")
                .takes_value(true)
                .default_value(""),
            ).arg(
            Arg::with_name("run_server")
            .long("run-server")
            .short("r")
            .help("Run a simple webserver on localhost, serving `outdir`, to test the generated posts")    
        ).arg(
            Arg::with_name("print_graph")
            .long("print-graph")
            .short("g")
            .help("print a graphviz graph at the end of processing, to visually check the post relationship structure")
        ).arg(
            Arg::with_name("no_html")
            .long("no-html")
            .short("n")
            .help("Don't actually generate the posts. Useful for just validating structure, or with -g to only print the graph.")
        ).get_matches()
}
fn unwraps_or_exits<T, E: Display + Debug>(t: Result<Vec<T>, Vec<E>>) -> Vec<T> {
    t.unwrap_or_else(|errors| {
        errors.iter().for_each(|e| error!("{}", e));
        std::process::exit(1)
    })
}
fn unwrap_or_exits<T, E: Display + Debug>(t: Result<T, Vec<E>>) -> T {
    t.unwrap_or_else(|errors| {
        errors.iter().for_each(|e| error!("{}", e));
        std::process::exit(1)
    })
}

fn unwrap_or_exit<T, E: Display + Debug>(t: Result<T, E>) -> T {
    t.unwrap_or_else(|e| {
        error!("{}", e);
        std::process::exit(1)
    })
}
fn copy_dir(src: &PathBuf, target: &PathBuf) -> std::io::Result<()> {
    if let Err(e) = create_dir(&target) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(e);
        }
    }
    for entry in read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let new_target = target.join(entry.file_name());

        if path.is_dir() {
            copy_dir(&path, &new_target)?;
        } else {
            copy(&path, &new_target)?;
        }
    }
    Ok(())
}

fn create_www(wwwdir: &PathBuf, cssdir: &PathBuf, force_del: bool) {
    if wwwdir.exists() && wwwdir.is_dir() {
        if force_del || Confirmation::new(format!("Delete {:?}?", wwwdir).as_str())
            .interact()
            .unwrap()
        {
            warn!("Removing directory {:?}", wwwdir);
            unwrap_or_exit(remove_dir_all(wwwdir));
        } else {
            error!("No output directory to work with");
            std::process::exit(1);
        }
    }
    unwrap_or_exit(create_dir_all(wwwdir));
    trace!("Moving {:?} to {:?}", cssdir, &wwwdir.join("css"));
    unwrap_or_exit(copy_dir(&cssdir, &wwwdir.join("css")));
}
fn run_webserver(wwwdir: &PathBuf) {
    info!("Running webserver on 127.0.0.1:3000 ...");
    let mut mount = mount::Mount::new();
    mount.mount("/", staticfile::Static::new(wwwdir));
    iron::Iron::new(mount).http("127.0.0.1:3000").unwrap();
}
fn main() {
    // only fails if another logger is initialized

    let args = argparse();
    let getval = |x| args.value_of(x).unwrap();
    let wwwdir = Path::new(getval("outdir")).to_path_buf();
    let templatedir = Path::new(getval("templatedir"));
    let cssdir = templatedir.join("css");
    let postdir = Path::new(getval("postdir"));
    let templateglob = format!("{}/jinja2/*", getval("templatedir"));
    let basepath = getval("basepath");

    let loglevel = match args.occurrences_of("v") {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    TermLogger::init(loglevel, Config::default()).unwrap();

    // read out the markdown to structs
    info!("Parsing posts..");
    let posts: Vec<_> = unwraps_or_exits(posts::get_posts(&postdir.to_path_buf()));

    // graph based on parents; we'll generate the symlinks from the graph.
    debug!("Constructing graph");
    let mut graph = Graph::new();

    unwrap_or_exits(graph.add_posts(&posts));
    for post in &posts {
        post.set_children_names(graph.get_children_names(&post));
        post.set_parent_names(graph.get_parent_names(&post));
    }

    if !args.is_present("no_html") {
        debug!("Fetching templates from {:?}", templateglob);
        let tera = get_templates(&templateglob);

        create_www(&wwwdir, &cssdir, args.is_present("delete_outdir"));
        // struct -> html
        info!("Generating html..");
        let post_templates = unwraps_or_exits(gen_posts_html(&tera, &posts, &graph, &basepath));
        // generate the actual files and symlinks
        debug!("Writing posts");
        unwrap_or_exit(create_posts(&wwwdir, &post_templates));
        // debug!("Writing symlinks");
        // unwrap_or_exit(create_symlinks(&wwwdir, &srcdir, &post_templates));
        // unwrap_or_exit(create_symlinks(&wwwdir, &srcdir, &graph));
        info!("Finished");
    }
    if args.is_present("run_server") {
        info!("Starting webserver");
        run_webserver(&wwwdir);
    }

    if args.is_present("print_graph") {
        println!("Use the following digraph on http://www.webgraphviz.com");
        println!("{}", graph.dot());
    }
}
