use errors::GraphError;
use petgraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::{EdgeRef, NodeIndexable};
use posts::{Category, Post, PostTypes};
use std::collections::HashMap;
#[derive(Debug)]
pub enum PostNode<'a> {
    Node(&'a PostTypes),
    Root(),
}
#[derive(Default)]
pub struct Graph<'a> {
    root: petgraph::graph::NodeIndex,
    pub graph: petgraph::Graph<PostNode<'a>, usize>,
    name_map: HashMap<&'a str, petgraph::graph::NodeIndex>,
}

impl<'a> Graph<'a> {
    pub fn new() -> Graph<'a> {
        let mut graph = petgraph::Graph::new();
        let name_map = HashMap::new();
        let root = graph.add_node(PostNode::Root());
        Graph {
            root,
            graph,
            name_map,
        }
    }

    pub fn add_posts(self: &mut Self, items: &'a [PostTypes]) -> Result<(), Vec<GraphError>> {
        // Add the posts to the graph
        let mut errors: Vec<GraphError> = Vec::new();
        for item in items {
            self.add_node(item);
        }

        // set the relationships based on stated parent relationship
        for item in items {
            match self.add_edge(item.name(), item.parents()) {
                Ok(()) => (),
                Err(x) => errors.push(x),
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    //TODO: Add sorting on names.
    pub fn get_child_cats(self: &Self, post: &'a PostTypes) -> Vec<&Category> {
        let idx = self.name_map[post.name()];
        self.graph
            .neighbors(idx)
            .map(|idx| &self.graph[idx])
            .map(|node| match node {
                PostNode::Node(PostTypes::Category(c)) => Some(c),
                _ => None,
            }).filter(Option::is_some)
            .map(Option::unwrap)
            .collect()
    }
    pub fn get_child_posts(self: &Self, post: &'a PostTypes) -> Vec<&Post> {
        let idx = self.name_map[post.name()];

        self.graph
            .neighbors(idx)
            .map(|idx| &self.graph[idx])
            .map(|node| match node {
                PostNode::Node(PostTypes::Post(p)) => Some(p),
                _ => None,
            }).filter(Option::is_some)
            .map(Option::unwrap)
            .collect()
    }

    pub fn get_children_names(self: &Self, post: &'a PostTypes) -> Vec<String> {
        // now do the inverse; read the defined relationships and determine the child-relationship
        // which we'll use for the post's links.

        let idx = self.name_map[post.name()];
        self.graph
            .neighbors(idx)
            .map(|s| self.ix_to_name(s).to_string())
            .collect()
    }

    pub fn add_node(self: &mut Self, item: &'a PostTypes) -> petgraph::graph::NodeIndex {
        let node = self.graph.add_node(PostNode::Node(item));

        for n in item.names() {
            self.name_map.insert(n, node);
        }
        node
    }

    fn find_paths(
        self: &Self,
        nx: NodeIndex,
        cur_route: &mut Vec<NodeIndex>,
        all_routes: &mut Vec<Vec<NodeIndex>>,
    ) {
        if cur_route.contains(&nx) {
            return;
        }
        cur_route.push(nx);
        for child in self.graph.neighbors(nx) {
            let mut new_route = cur_route.clone();
            self.find_paths(child, &mut new_route, all_routes);
            if cur_route.len() != new_route.len() {
                all_routes.push(new_route);
            }
        }
    }
    fn ix_to_name(self: &Self, ix: NodeIndex) -> &str {
        match self.graph[ix] {
            PostNode::Node(n) => n.name(),
            PostNode::Root() => "Root",
        }
    }
    fn ixs_to_name(self: &Self, ixs: &[NodeIndex]) -> Vec<&str> {
        ixs.iter().map(|&n| self.ix_to_name(n)).collect()
    }

    pub fn find_all_paths(self: &Self) -> Vec<(&str, Vec<&str>)> {
        let mut all_routes: Vec<_> = self.graph.neighbors(self.root).map(|nx| vec![nx]).collect();

        for nx in self.graph.neighbors(self.root) {
            let mut routes = vec![];
            self.find_paths(nx, &mut vec![], &mut routes);
            all_routes.append(&mut routes);
        }

        // (post, route_to_it)
        all_routes
            .iter()
            .map(|r| {
                (
                    self.ix_to_name(r[r.len() - 1]),
                    self.ixs_to_name(&r[0..r.len() - 1]),
                )
            }).collect()
    }

    pub fn add_edge(self: &mut Self, name: &str, parentlist: &[String]) -> Result<(), GraphError> {
        let child = match self.name_map.get(name) {
            Some(&c) => c,
            None => panic!("Edge was added before the Node itself was: {}", name),
        };
        // add to root node
        if parentlist.is_empty() {
            self.graph.add_edge(self.root, child, 0);
            return Ok(());
        }
        let map = &mut self.name_map;
        let graph = &mut self.graph;
        let (parents, errors): (Vec<_>, Vec<_>) = parentlist
            .iter()
            .map(|p| map.get(&p as &str))
            .partition(Option::is_some);

        if !errors.is_empty() {
            let missing_parents: Vec<_> = errors
                .iter()
                .zip(parentlist.iter())
                .map(|(_, ref pname)| pname.to_string())
                .collect();
            return Err(GraphError::MissingEdgeError(
                name.to_string(),
                missing_parents,
            ));
        }

        // Silently ignore multiple references to the same parent, by a single post
        let mut parents: Vec<_> = parents.iter().map(|x| x.unwrap()).collect();
        parents.sort_unstable();
        parents.dedup();

        for parent in parents {
            graph.add_edge(*parent, child, 0);
        }
        Ok(())
    }

    // simplified graphviz code, based on petgraph's graph conversion source
    // only prints out name() for the node labels
    pub fn dot(self: &Self) -> String {
        static INDENT: &'static str = "    ";
        let mut f = vec![];
        f.push("digraph {{".to_string());
        for ix in self.graph.node_indices() {
            let name = match &self.graph[ix] {
                PostNode::Node(n) => match n {
                    PostTypes::Category(_) => format!("Category({})", n.name()),
                    PostTypes::Post(_) => format!("Post({})", n.name()),
                },
                PostNode::Root() => "ROOT".to_string(),
            };

            f.push(format!("{}{} [label=\"{}\"]", INDENT, ix.index(), name));
        }

        for edge in self.graph.edge_references() {
            f.push(format!(
                "{}{} -> {}",
                INDENT,
                self.graph.to_index(edge.source()),
                self.graph.to_index(edge.target())
            ));
        }
        f.push("}}".to_string());
        f.join("\n")
    }
}
