# redsystem
Static blog generator with a digraph structure (ie cycles allowed)

Any blog post/category can be a child of any post/category. Multiple paths can lead to the same post. The path taken is **not** recorded anywhere. JS is currently not used, and at no point should ever be required. 

The files hosted in this repo generate the website https://setr.github.io/redsystem/.

## Default Directory Structure

```
.
├── redsystem
├── posts/
├── templates/
└── www/
```
`Posts/` stores your articles.

`templates/` stores the html jinja2 templates and css. In the future, `templates/images/` will store your article images.

`www/` stores the output, which can be copied to your static webserver as-is.

Currently, how you organize articles within the `Posts/` directory makes no difference to `redsystem`. Specifically, the `dirname` field currently has no association with the directory the post/category was found in. Posts will only be read if they have the extension `.toml`.


## Usage
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
    -r, --run-server      Run a simple webserver on localhost, serving `outdir`, to test the generated posts
    -v, --verbose         Use verbose output. Repeat to increase verbosity, up to 3 times.
    -V, --version         Prints version information

OPTIONS:
    -b, --base-path <basepath>       Base path to set in the html, if you're not hosting from root. [default: ]
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

Currently Categories can have body text, but its html template doesn't do anything with it.
```
Metadata:
    [Required] type: "Post"
        Determines html template used, and possible metadata fields.
    [Required] name: String
        Canonical name of the document. Must be unique across all posts.
    [Optional] dirname: String
        The directory from root which will contain this. Acts as a namespace, and appears in the URL when visiting the page. ie "Artists" or "Artists/Tokyo". 
        This is the canonical root directory of a node.
    [Optional] aliases: [String]
        Alternative names that this post can be referenced by. Must be unique across all posts in the `dirname` namespace.
    [Optional] parents: [String]
        List of parent nodes, referenced by name/alias. Duplicate references to the same parent will be ignored.
        If no parents are listed, or the parent "INDEX" exists, it will be attached to the implicit index node (which produces index.html).
        Parents must be listed with the full path. ie if `Star Control` has alias `sc` and has dirname `Category`, then it be referenced as a parent with "Category/sc" or "Category/Star Control"
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
    [Optional] dirname: String
        The directory from root which will contain this. Acts as a namespace, and appears in the URL when visiting the page. ie "Artists" or "Artists/Tokyo". 
        This is the canonical root directory of a node.
    [Optional] aliases: [String]
        Alternative names that this post can be referenced by. Must be unique across all posts in the `dirname` namespace.
    [Optional] parents: [String]
        List of parent nodes, referenced by name/alias. Duplicate references to the same parent will be ignored.
        If no parents are listed, or the parent "INDEX" exists, it will be attached to the implicit index node (which produces index.html).
        Parents must be listed with the full path. ie if `Star Control` has alias `sc` and has dirname `Category`, then it be referenced as a parent with "Category/sc" or "Category/Star Control"
```
Note that the template used, and the required information for it, is determined by the `type`. Currently `type` can be either "Post" or "Category", where Post denotes something (ie a game), while Category denotes a group of things. 

Note that they can reference each other (using the parents field) arbitrarily; that is, a post can be the parent of many categories, and a category can be the parent of many posts, or category-\>category, or whatever combination you wish. The only special node is the index (root) node.

## Examples

### Example post
```
type = "Post"
name = 'Star Control: Famous Battles of the Ur-Quan Conflict, Volume IV'
aliases = ["Star Control"]
image = "http://4.bp.blogspot.com/-uJiRZMgyuQ0/UQVwZc-XwYI/AAAAAAAAArI/N7rhTIeb2-Y/s1600/36313-star-control-amiga-screenshot-the-syreen-penetrators-1.gif"
year = "1990"
parents = ["SciFi"]
---
**Star Control: Famous Battles of the Ur-Quan Conflict Volume IV** was developed by Fred Ford and Paul Reiche III. It was released for the PC in 1990 and (in a somewhat cut-down form, and called simply Star Control) for the Commodore 64 in 1991 by Accolade. 

A port for the Sega Genesis was released by Ballistic in that same year. The DOS version is available from Good Old Games.
```
### Another Example post
```
type = "Post"
name = 'Star Control II - The Urquan Masters'
aliases = ["Star Control II", "Star Control 2"]
image = "https://draginol.stardock.net/images2018/The-art-of-Star-Control_C786/image.png"
year = "1992"
parents = ["SciFi", "Star Control"]
---
**Star Control II: The Ur-Quan Masters** is a science fiction video game, a sequel to Star Control. It was developed by Toys for Bob and originally published by Accolade in 1992 for MS-DOS. 

It was later ported to the 3DO by Crystal Dynamics in 1994 with an enhanced multimedia presentation, allowed by the CD technology.
```

### Example category
```
type="Category"
name="Science Fiction"
aliases=["SciFi", "Sci-Fi"]
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

### Hosting on github's gh-page
```
redsystem -f -o www -b "/redsystem"
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
