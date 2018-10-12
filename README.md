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

## Post Structure
```
[metadata]
---
[body text]
```
Metadata is expected in TOML structure.

The dividing line `---` is required if body text exists (otherwise redsystem will attempt to parse the body text as metadata, and fail).

Body text is parsed as standard markdown.


```
Metadata:
    [Required] type: "Post"
        Determines html template used, and possible metadata fields.
    [Required] name: String
        Canonical name of the document. Must be unique across all posts.
    [Optional] parents: [String]
        List of parent nodes, referenced by name/alias. Duplicate references to the same parent will be ignored.
    [Optional] aliases: [String]
        Alternative names that this post can be referenced by. Must be unique across all posts.
    [Optional] description: String
        Short one-line description of the post
    [Optional] image: String
        URL for post's main image, or name of the image stored in templates/images.
    [Optional] year: String
        Year of topic's creation
    [Optional] dl_url: String
        URL to download the topic.
or
    [Required] type: "Category"
        Determines html template used, and possible metadata fields.
    [Required] name: String
        Canonical name of the document. Must be unique across all posts.
    [Optional] parents: [String]
        List of parent nodes, referenced by name/alias. Duplicate references to the same parent will be ignored.
    [Optional] aliases: [String]
        Alternative names that this post can be referenced by. Must be unique across all posts.
```
Note that the template used, and the required information for it, is determined by the `type`. Currently `type` can be either "Post" or "Category", where Post denotes something (ie a game), while Category denotes a group of things. Note that they can reference each other (using the parents field) arbitrarily; that is, a post can be the parent of many categories, and a category can be the parent of many posts, or category-\>category, or whatever combination you wish.

## Example Usage

### Example post
```
type="Post"
name = "Omega Boost"
aliases=["Omega", "omega"]
year = "1993"
description = "Omega Boost"
image = "https://gamefaqs.akamaized.net/box/8/6/6/5866_front.jpg"
dl_url = "url.org/#64zx79sYfq2TD82vTexI19DKua4Ns8YXMHBYWkrsCMpUZRaj1QLAUjKxIApEE1cQGt8wviSh8pH58N623HviJiFq7T4oFlOZCMov"
parents = ["misc studio", "shoji kawamori", "cyber", "mecha"]
---
Lorem ipsum dolor sit amet, magna iusto senserit vel in, ignota eirmod officiis cu quo, posidonium necessitatibus no eum. Cu mea diceret mediocrem dissentias, sed partem recusabo invenire ut. Ex adipisci tacimates pri. Vide nemore molestie ad quo. Mea quidam regione antiopam te. Eius iracundia eam ad, putent nominavi ex eos.

Lorem ipsum dolor sit amet, magna iusto senserit vel in, ignota eirmod officiis cu quo, posidonium necessitatibus no eum. Cu mea diceret mediocrem dissentias, sed partem recusabo invenire ut. Ex adipisci tacimates pri. Vide nemore molestie ad quo. Mea quidam regione antiopam te. Eius iracundia eam ad, putent nominavi ex eos.
```
### Another Example post
```
type = "Post"
name = 'post1'
parents = ["catA", "catB", "Omega"]
```

### Example category
```
type="Category"
name="catA"
---
slkjsdlfdjsl
```

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

