use std::io;
use std::path::{Path, PathBuf};
use toml;
quick_error! {
    #[derive(Debug)]
    pub enum IOError {
        IOError(err: io::Error, path: PathBuf){
            display("Error with path {:?}:  {}", path, err)
            context(path: &'a PathBuf, err: io::Error)
                -> (err, path.to_path_buf())
        }
        IOError2(err: io::Error, path: PathBuf){
            display("Error with path {:?}:  {}", path, err)
            context(path: &'a Path, err: io::Error)
                -> (err, path.to_path_buf())
        }
        missing_directory(dir: PathBuf){
            display("Expected directory {:?} is missing", dir)
        }

        missing_post_header( file: PathBuf) {
            display("The post {:?} is missing its header", file)
        }
        invalid_header(err: toml::de::Error, file: PathBuf){
            display("The post {:?} has an invalid header: {}", file, err)
        }
        duplicate_name(name: String, post1: PathBuf, post2:PathBuf){
            display("Duplicate names: Post {:?} and {:?} share the name/alias - {}", post1, post2, name)
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum GraphError {
        MissingEdgeError(child: String, missing_parents: Vec<String>) {
            description("The given Node is a child of Nodes that do not exist")
            display(r#"The post {} claims non-existent parents: {:?}"#, child, missing_parents)

        }

    }
}
