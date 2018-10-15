use errors;
use errors::IOError::*;
use quick_error::ResultExt;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use toml;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Category {
    pub name: String,
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub dirname: String, // associated directory
    #[serde(skip_deserializing)]
    pub body: String,
    #[serde(skip_deserializing)]
    pub children: RefCell<Vec<String>>,
    #[serde(skip_deserializing)]
    pub parent_names: RefCell<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Post {
    pub name: String,
    #[serde(default)]
    pub parents: Vec<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub dirname: String, // associated directory
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub year: String,
    #[serde(default)]
    pub dl_url: String,

    #[serde(skip_deserializing)]
    pub body: String,
    #[serde(skip_deserializing)]
    pub children: RefCell<Vec<String>>,
    #[serde(skip_deserializing)]
    pub parent_names: RefCell<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PostTypes {
    Post(Post),
    Category(Category),
}

impl PostTypes {
    pub fn name(&self) -> String {
        match self {
            PostTypes::Post(p) => self.withdir(&p.name),
            PostTypes::Category(c) => self.withdir(&c.name),
        }
    }

    pub fn names(&self) -> Vec<String> {
        match self {
            PostTypes::Post(p) => {
                let mut names: Vec<_> = p.aliases.iter().map(|s| self.withdir(s)).collect();
                names.push(self.name());
                names
            }
            PostTypes::Category(c) => {
                let mut names: Vec<_> = c.aliases.iter().map(|s| self.withdir(s)).collect();
                names.push(self.name());
                names
            }
        }
    }
    fn get_dir(&self) -> &str {
        match self {
            PostTypes::Post(p) => &p.dirname,
            PostTypes::Category(c) => &c.dirname,
        }
    }

    fn withdir(&self, name: &str) -> String {
        if self.get_dir().is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", self.get_dir(), name)
        }
    }
    pub fn parents(&self) -> &Vec<String> {
        match self {
            PostTypes::Post(p) => &p.parents,
            PostTypes::Category(c) => &c.parents,
        }
    }
    pub fn set_children_names(&self, children: Vec<String>) {
        match self {
            PostTypes::Post(p) => p.children.borrow_mut().extend(children),
            PostTypes::Category(c) => c.children.borrow_mut().extend(children),
        }
    }
    pub fn set_parent_names(&self, parents: Vec<String>) {
        match self {
            PostTypes::Post(p) => p.parent_names.borrow_mut().extend(parents),
            PostTypes::Category(c) => c.parent_names.borrow_mut().extend(parents),
        }
    }
}

pub fn get_post(filepath: &PathBuf) -> Result<PostTypes, errors::IOError> {
    trace!("Parsing post {:?}", filepath);
    let mut f = File::open(filepath).context(filepath)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents).context(filepath)?;
    let mut split = contents.splitn(2, "\n---\n");
    let tomlcfg = match split.next() {
        Some(s) => s.to_string(),
        None => return Err(missing_post_header(filepath.to_path_buf())),
    };

    let body = match split.next() {
        Some(s) => s.to_string(),
        None => String::new(),
    };

    match toml::from_str::<PostTypes>(&tomlcfg) {
        Ok(s) => match s {
            PostTypes::Post(mut p) => {
                p.body = body;
                Ok(PostTypes::Post(p))
            }
            PostTypes::Category(mut c) => {
                c.body = body;
                Ok(PostTypes::Category(c))
            }
        },
        Err(e) => Err(invalid_header(e, filepath.to_path_buf())),
    }
}

fn find_files<F: Fn(&PathBuf) -> bool>(
    dir: &PathBuf,
    filepaths: &mut Vec<PathBuf>,
    filter: &F,
) -> Result<(), errors::IOError> {
    for entry in fs::read_dir(dir).context(dir)? {
        let path = entry.context(dir)?.path();
        if path.is_dir() {
            find_files(&path, filepaths, filter)?;
        } else if filter(&path) {
            filepaths.push(path);
        }
    }
    Ok(())
}

pub fn get_posts(postdir: &PathBuf) -> Result<Vec<PostTypes>, Vec<errors::IOError>> {
    let mut filenames = vec![];

    if find_files(postdir, &mut filenames, &|p| match p.extension() {
        Some(ext) => ext == "toml",
        None => false,
    }).is_err()
    {
        return Err(vec![missing_directory(postdir.to_path_buf())]);
    }
    info!("Found {} posts", filenames.len());
    let (posts, errors): (Vec<_>, Vec<_>) = filenames
        .iter()
        .map(|ref f| get_post(&f.to_path_buf()))
        .partition(Result::is_ok);

    if errors.is_empty() {
        let finalposts: Vec<_> = posts.into_iter().map(Result::unwrap).collect();
        let mut errors = vec![];
        // all this, just to verify that posts have unique names/aliases
        // check has to be done at this point, while we can still map posts back to the original filename it came from
        // otherwise, we'll need to store and return it
        for (n, (f, p)) in filenames.iter().zip(&finalposts).enumerate() {
            let names = p.names();
            let nameset: HashSet<_> = names.iter().collect();
            for (f2, p2) in filenames.iter().zip(&finalposts).skip(n + 1) {
                let names2 = p2.names();
                let nameset2: HashSet<_> = names2.iter().collect();
                let mut collisions = nameset
                    .intersection(&nameset2)
                    .map(|n| duplicate_name((*n).to_string(), f.to_path_buf(), f2.to_path_buf()))
                    .collect();
                errors.append(&mut collisions);
            }
        }
        if errors.is_empty() {
            Ok(finalposts)
        } else {
            Err(errors)
        }
    } else {
        Err(errors.into_iter().map(Result::unwrap_err).collect())
    }
}
