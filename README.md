# redsystem
Static blog generator with cyclical digraph structure

```
test% target/debug/redsystem -h
redsystem 0.1.0
setr <nokhand@gmail.com>
Blog generator with cyclical graph structure

USAGE:
    redsystem [FLAGS] [OPTIONS]

FLAGS:
    -f, --force_delete    delete outdir without prompting
    -h, --help            Prints help information
    -n, --no_html         Don't actually generate the posts. Useful for just validating structure, or with -g to only
                          print the graph.
    -g, --print_graph     print a graphviz graph at the end of processing, to visually check the post relationship
                          structure
    -r, --run_server      Run a simple webserver on localhost, serving `outdir`, to test the generated posts
    -v                    Verbosity
    -V, --version         Prints version information

OPTIONS:
    -o, --outdir <outdir>            Directory to write generated files to [default: ./www]
    -p, --posts <postdir>            Directory to fetch content files from [default: ./posts]
    -t, --templates <templatedir>    Directory to fetch html templates and css from [default: ./templates]
```
