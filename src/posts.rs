use errors;
use errors::IOError::*;
use quick_error::ResultExt;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::fmt;
use serde::de::{Visitor, SeqAccess, value, Deserialize, Deserializer};
use toml;

#[derive(Serialize, Debug, Default, Clone)]
pub struct TeraNextPost {
    pub path: String,
    pub title: String,
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Category {
    pub name: String,
    #[serde(default)]
    pub title: String,
    #[serde(default, deserialize_with = "string_or_vec", rename="parent")]
    pub parents: Vec<String>,
    #[serde(default, deserialize_with = "string_or_vec", rename="alias")]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub dirname: String, // associated directory
    #[serde(skip_deserializing)]
    pub body: String,
    #[serde(skip_deserializing)]
    pub children: RefCell<Vec<TeraNextPost>>,
    #[serde(skip_deserializing)]
    pub parent_names: RefCell<Vec<TeraNextPost>>,
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Post {
    pub name: String,
    #[serde(default)]
    pub title: String,
    #[serde(default, deserialize_with = "string_or_vec", rename="parent")]
    pub parents: Vec<String>,
    #[serde(default, deserialize_with = "string_or_vec", rename="alias")]
    pub aliases: Vec<String>,
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
    pub children: RefCell<Vec<TeraNextPost>>,
    #[serde(skip_deserializing)]
    pub parent_names: RefCell<Vec<TeraNextPost>>,
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
    pub fn basename(&self) -> String {
        match self {
            PostTypes::Post(p) => p.name.to_string(),
            PostTypes::Category(c) => c.name.to_string(),
        }
    }
    pub fn title(&self) -> String {
        match self {
            PostTypes::Post(p) => p.title.to_string(),
            PostTypes::Category(c) => c.title.to_string(),
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
    pub fn set_children_names(&self, children: Vec<(String, String)>) {
        let chs: Vec<_> = children
            .iter()
            .map(|(path, title)| TeraNextPost {
                path: path.to_string(),
                title: title.to_string(),
            }).collect();
        match self {
            PostTypes::Post(p) => p.children.borrow_mut().extend(chs),
            PostTypes::Category(c) => c.children.borrow_mut().extend(chs),
        }
    }
    pub fn set_parent_names(&self, parents: Vec<(String, String)>) {
        let chs: Vec<_> = parents
            .iter()
            .map(|(path, title)| TeraNextPost {
                path: path.to_string(),
                title: title.to_string(),
            }).collect();
        match self {
            PostTypes::Post(p) => p.parent_names.borrow_mut().extend(chs),
            PostTypes::Category(c) => c.parent_names.borrow_mut().extend(chs),
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

    // add the body text; title defaults to name.
    match toml::from_str::<PostTypes>(&tomlcfg) {
        Ok(s) => match s {
            PostTypes::Post(mut p) => {
                p.body = body;
                p.title = match p.title.as_str() {
                    "" => p.name.clone(),
                    _ => p.title,
                };
                Ok(PostTypes::Post(p))
            }
            PostTypes::Category(mut c) => {
                c.body = body;
                c.title = match c.title.as_str() {
                    "" => c.name.clone(),
                    _ => c.title,
                };
                Ok(PostTypes::Category(c))
            }
        },
        Err(e) => Err(invalid_header(e, filepath.to_path_buf())),
    }
}

fn string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where D: Deserializer<'de>
{
    struct StringOrVec;

    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where E: serde::de::Error
        {
            Ok(vec![s.to_owned()])
        }
        fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
            where E: serde::de::Error
        {
            Ok(vec![s.to_owned()])
        }

        fn visit_seq<S>(self, seq: S) -> Result<Self::Value, S::Error>
            where S: SeqAccess<'de>
        {
            Deserialize::deserialize(value::SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(StringOrVec)
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
pub fn get_fakeposts(posts: &[PostTypes]) -> Vec<PostTypes> {
    let names: HashSet<_> = posts.iter().flat_map(|item| item.names()).collect();

    posts
        .iter()
        .flat_map(|item| item.parents())
        .filter(|&p| !names.contains(p))
        .map(|parent| {
            PostTypes::Post(Post {
                name: parent.clone(),
                ..Default::default()
            })
        }).collect()
}
