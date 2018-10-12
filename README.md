# redsystem
Static blog generator with cyclical digraph structure

```
test% redsystem -h
redsystem 0.1.0
setr <nokhand@gmail.com>
Static blog generator with cyclical digraph structure

USAGE:
    redsystem [FLAGS] [OPTIONS]

FLAGS:
    -f, --force-delete    delete outdir without prompting
    -h, --help            Prints help information
    -n, --no-html         Don't actually generate the posts. Useful for just validating structure, or with -g to only
                          print the graph.
    -g, --print-graph     print a graphviz graph at the end of processing, to visually check the post relationship
                          structure
    -r, --run_server      Run a simple webserver on localhost, serving `outdir`, to test the generated posts
    -v, --verbose         Use verbose output. Repeat to increase verbosity, up to 3 times.
    -V, --version         Prints version information

OPTIONS:
    -o, --outdir <outdir>            Directory to write generated files to [default: ./www]
    -p, --posts <postdir>            Directory to fetch content files from [default: ./posts]
    -t, --templates <templatedir>    Directory to fetch html templates and css from [default: ./templates]
```

## Example Usage

### Create html files, remove /www directory, and run a simple webserver to take a look at it
```
test% redsystem -fr
20:12:36 [INFO] Parsing posts..
20:12:36 [INFO] Found 5 posts
20:12:36 [WARN] Removing directory "./www"
20:12:36 [INFO] Generating html..
20:12:36 [INFO] Finished
20:12:36 [INFO] Starting webserver
20:12:36 [INFO] Running webserver on 127.0.0.1:3000 ...
```

### Only print Graph
```
test% redsystem -gn
20:10:38 [INFO] Parsing posts..
20:10:38 [INFO] Found 5 posts
Use the following digraph on http://www.webgraphviz.com
digraph {{
    0 [label="ROOT"]
    1 [label="Category(catA)"]
    2 [label="Category(catB)"]
    3 [label="Post(Omega Boost)"]
    4 [label="Post(post1)"]
    5 [label="Post(post2)"]
    0 -> 1
    1 -> 2
    4 -> 2
    5 -> 2
    2 -> 3
    1 -> 4
    2 -> 4
    3 -> 4
    4 -> 5
}}
```

