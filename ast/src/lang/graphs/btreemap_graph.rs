use super::{graph::Graph, *};
use crate::lang::{Function, FunctionCall, Lang};
use crate::utils::create_node_key;
use anyhow::Result;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BTreeMapGraph {
    pub nodes: BTreeMap<String, Node>,
    pub edges: BTreeMap<(String, String), EdgeType>,
}

impl Graph for BTreeMapGraph {
    fn new() -> Self {
        BTreeMapGraph {
            nodes: BTreeMap::new(),
            edges: BTreeMap::new(),
        }
    }

    fn with_capacity(_nodes: usize, _edges: usize) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    fn extend_graph(&mut self, other: Self) {
        self.nodes.extend(other.nodes);
        self.edges.extend(other.edges);
    }

    fn get_graph_size(&self) -> (u32, u32) {
        (self.nodes.len() as u32, self.edges.len() as u32)
    }
    fn add_edge(&mut self, edge: Edge) {
        if let (Some(source_node), Some(target_node)) = (
            self.find_node_from_node_ref(edge.source),
            self.find_node_from_node_ref(edge.target),
        ) {
            let source_key = create_node_key(source_node);
            let target_key = create_node_key(target_node);
            self.edges.insert((source_key, target_key), edge.edge);
        }
    }

    fn find_nodes_by_name(&self, node_type: NodeType, name: &str) -> Vec<NodeData> {
        let prefix = format!("{:?}-{}", node_type, name.to_lowercase());

        self.nodes
            .range(prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&prefix))
            .map(|(_, node)| node.node_data.clone())
            .collect()
    }
    fn find_node_by_name_in_file(
        &self,
        node_type: NodeType,
        name: &str,
        file: &str,
    ) -> Option<NodeData> {
        let prefix = format!(
            "{:?}-{}-{}",
            node_type,
            name.to_lowercase(),
            file.to_lowercase()
        );

        self.nodes
            .range(prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&prefix))
            .map(|(_, node)| node.node_data.clone())
            .next()
    }

    fn add_node_with_parent(
        &mut self,
        node_type: NodeType,
        node_data: NodeData,
        parent_type: NodeType,
        parent_file: &str,
    ) {
        let node = Node::new(node_type.clone(), node_data.clone());
        let node_key = create_node_key(node.clone());

        let prefix = format!("{:?}-", parent_type);
        if let Some((parent_key, _)) = self
            .nodes
            .range(prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&prefix))
            .find(|(_, n)| n.node_data.file == parent_file)
        {
            self.edges
                .insert((parent_key.clone(), node_key), EdgeType::Contains);
        }
    }

    fn create_filtered_graph(&self, final_filter: &[String]) -> Self {
        let mut filtered = Self::new();

        for (key, node) in &self.nodes {
            if node.node_type == NodeType::Repository || final_filter.contains(&node.node_data.file)
            {
                filtered.nodes.insert(key.clone(), node.clone());
            }
        }

        for ((src, dst), edge_type) in &self.edges {
            if let (Some(src_node), Some(dst_node)) = (self.nodes.get(src), self.nodes.get(dst)) {
                if final_filter.contains(&src_node.node_data.file)
                    || final_filter.contains(&dst_node.node_data.file)
                {
                    filtered
                        .edges
                        .insert((src.clone(), dst.clone()), edge_type.clone());
                }
            }
        }

        filtered
    }

    fn find_nodes_in_range(&self, node_type: NodeType, row: u32, file: &str) -> Option<NodeData> {
        let prefix = format!("{:?}-", node_type);

        self.nodes
            .range(prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&prefix))
            .find(|(_, node)| {
                node.node_data.file == file
                    && (node.node_data.start as u32) <= row
                    && (node.node_data.end as u32) >= row
            })
            .map(|(_, node)| node.node_data.clone())
    }

    fn find_node_at(&self, node_type: NodeType, file: &str, line: u32) -> Option<NodeData> {
        let prefix = format!("{:?}-", node_type);

        self.nodes
            .range(prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&prefix))
            .find(|(_, node)| node.node_data.file == file && node.node_data.start == line as usize)
            .map(|(_, node)| node.node_data.clone())
    }

    fn find_node_by_name_and_file_end_with(
        &self,
        node_type: NodeType,
        name: &str,
        suffix: &str,
    ) -> Option<NodeData> {
        let prefix = format!("{:?}-{}", node_type, name.to_lowercase());
        self.nodes
            .range(prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&prefix))
            .find(|(_, node)| node.node_data.file.ends_with(suffix))
            .map(|(_, node)| node.node_data.clone())
    }

    fn find_nodes_by_file_ends_with(&self, node_type: NodeType, file: &str) -> Vec<NodeData> {
        let prefix = format!("{:?}-", node_type);
        self.nodes
            .range(prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&prefix))
            .filter(|(_, node)| node.node_data.file.ends_with(file))
            .map(|(_, node)| node.node_data.clone())
            .collect()
    }
    fn find_source_edge_by_name_and_file(
        &self,
        edge_type: EdgeType,
        target_name: &str,
        target_file: &str,
    ) -> Option<NodeKeys> {
        for ((src_key, dst_key), edge) in &self.edges {
            if edge == &edge_type {
                if let (Some(src_node), Some(dst_node)) =
                    (self.nodes.get(src_key), self.nodes.get(dst_key))
                {
                    if dst_node.node_data.name == target_name
                        && dst_node.node_data.file == target_file
                    {
                        return Some(NodeKeys::from(&src_node.node_data));
                    }
                }
            }
        }
        None
    }
    fn add_instances(&mut self, instances: Vec<NodeData>) {
        for inst in instances {
            if let Some(of) = &inst.data_type {
                if let Some(class_node) = self.find_nodes_by_name(NodeType::Class, of).first() {
                    let node = Node::new(NodeType::Instance, inst.clone());
                    let node_key = create_node_key(node.clone());
                    self.nodes.insert(node_key.clone(), node);

                    let class_key =
                        format!("{:?}-{}", NodeType::Class, class_node.name.to_lowercase());
                    self.edges.insert((node_key, class_key), EdgeType::Of);
                }
            }
        }
    }

    fn add_functions(&mut self, functions: Vec<Function>) {
        for (node, method_of, reqs, dms, trait_operand, return_types) in functions {
            let func_node = Node::new(NodeType::Function, node.clone());
            let func_key = create_node_key(func_node.clone());
            self.nodes.insert(func_key.clone(), func_node);

            let file_prefix = format!("{:?}-{}", NodeType::File, node.file.to_lowercase());
            if let Some((file_key, _)) = self.nodes.range(file_prefix..).next() {
                self.edges
                    .insert((file_key.clone(), func_key.clone()), EdgeType::Contains);
            }
            // Add method_of edge if present
            if let Some(p) = method_of {
                self.add_edge(p.into());
            }

            // Add trait operand edge if present
            if let Some(to) = trait_operand {
                self.add_edge(to.into());
            }

            // Add return type edges
            for rt in return_types {
                self.add_edge(rt);
            }
            for r in reqs {
                let req_node = Node::new(NodeType::Request, r.clone());
                self.nodes
                    .insert(create_node_key(req_node.clone()), req_node);

                let edge = Edge::calls(
                    NodeType::Function,
                    &node,
                    NodeType::Request,
                    &r,
                    CallsMeta {
                        call_start: r.start,
                        call_end: r.end,
                        operand: None,
                    },
                );
                self.add_edge(edge);
            }

            // Add datamodel edges
            for dm in dms {
                self.add_edge(dm);
            }
        }
    }

    fn add_page(&mut self, page: (NodeData, Option<Edge>)) {
        let (page_data, edge_opt) = page;
        let page_node = Node::new(NodeType::Page, page_data.clone());
        let page_key = create_node_key(page_node.clone());
        self.nodes.insert(page_key.clone(), page_node);

        if let Some(edge) = edge_opt {
            self.add_edge(edge);
        }
    }

    fn add_pages(&mut self, pages: Vec<(NodeData, Vec<Edge>)>) {
        for (page_data, edges) in pages {
            let page_node = Node::new(NodeType::Page, page_data.clone());
            let page_key = create_node_key(page_node.clone());
            self.nodes.insert(page_key.clone(), page_node);

            for edge in edges {
                self.add_edge(edge);
            }
        }
    }

    fn find_endpoint(&self, name: &str, file: &str, verb: &str) -> Option<NodeData> {
        let prefix = format!("{:?}-{}-{}", NodeType::Endpoint, name, file);

        self.nodes
            .range(prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&prefix))
            .find(|(_, node)| node.node_data.meta.get("verb") == Some(&verb.to_string()))
            .map(|(_, node)| node.node_data.clone())
    }

    fn add_endpoints(&mut self, endpoints: Vec<(NodeData, Option<Edge>)>) {
        for (endpoint_data, handler_edge) in endpoints {
            if endpoint_data.meta.get("handler").is_some() {
                let default_verb = "".to_string();
                let verb = endpoint_data.meta.get("verb").unwrap_or(&default_verb);

                if self
                    .find_endpoint(&endpoint_data.name, &endpoint_data.file, verb)
                    .is_some()
                {
                    continue;
                }

                let endpoint_node = Node::new(NodeType::Endpoint, endpoint_data.clone());
                let endpoint_key = create_node_key(endpoint_node.clone());
                self.nodes.insert(endpoint_key.clone(), endpoint_node);

                if let Some(edge) = handler_edge {
                    self.add_edge(edge);
                }
            }
        }
    }

    fn add_test_node(&mut self, test_data: NodeData, test_type: NodeType, test_edge: Option<Edge>) {
        self.add_node_with_parent(
            test_type,
            test_data.clone(),
            NodeType::File,
            &test_data.file,
        );

        if let Some(edge) = test_edge {
            self.add_edge(edge);
        }
    }

    fn add_calls(&mut self, calls: (Vec<FunctionCall>, Vec<FunctionCall>, Vec<Edge>)) {
        let (funcs, tests, int_tests) = calls;

        for (fc, ext_func) in funcs {
            if let Some(ext_nd) = ext_func {
                if let Some(source_node) =
                    self.find_node_from_node_key(NodeType::Function, fc.source)
                {
                    let source_key = create_node_key(source_node);
                    let ext_node = Node::new(NodeType::Function, ext_nd.clone());
                    let ext_key = create_node_key(ext_node.clone());

                    if !self.nodes.contains_key(&ext_key) {
                        self.nodes.insert(ext_key.clone(), ext_node);
                    }
                    self.edges.insert((source_key, ext_key), EdgeType::Uses);
                }
            } else {
                if let (Some(source_node), Some(target_node)) = (
                    self.find_node_from_node_key(NodeType::Function, fc.source.clone()),
                    self.find_node_from_node_key(NodeType::Function, fc.target),
                ) {
                    let source_key = create_node_key(source_node);
                    let target_key = create_node_key(target_node);
                    self.edges.insert(
                        (source_key, target_key),
                        EdgeType::Calls(CallsMeta {
                            call_start: fc.call_start,
                            call_end: fc.call_end,
                            operand: fc.operand,
                        }),
                    );
                }
            }
        }

        // Add test function calls
        for (tc, ext_func) in tests {
            if let Some(ext_nd) = ext_func {
                if let Some(source_node) =
                    self.find_node_from_node_key(NodeType::Function, tc.source)
                {
                    let source_key = create_node_key(source_node);
                    let ext_node = Node::new(NodeType::Function, ext_nd.clone());
                    let ext_key = create_node_key(ext_node.clone());

                    if !self.nodes.contains_key(&ext_key) {
                        self.nodes.insert(ext_key.clone(), ext_node);
                    }
                    self.edges.insert((source_key, ext_key), EdgeType::Uses);
                }
            }
        }

        // Add integration test edges
        for edge in int_tests {
            self.add_edge(edge);
        }
    }
    fn process_endpoint_groups(&mut self, eg: Vec<NodeData>, lang: &Lang) -> Result<()> {
        // Collect all updates we need to make
        let mut updates = Vec::new();

        for group in eg {
            if let Some(g) = group.meta.get("group") {
                if let Some(gf) = self.find_nodes_by_name(NodeType::Function, g).first() {
                    for q in lang.lang().endpoint_finders() {
                        let endpoints_in_group = lang.get_query_opt::<Self>(
                            Some(q),
                            &gf.body,
                            &gf.file,
                            NodeType::Endpoint,
                        )?;

                        for end in endpoints_in_group {
                            let prefix = format!("{:?}-{}", NodeType::Endpoint, end.name);
                            if let Some((key, node)) = self
                                .nodes
                                .range(prefix.clone()..)
                                .take_while(|(k, _)| k.starts_with(&prefix))
                                .next()
                            {
                                let new_endpoint =
                                    format!("{}{}", group.name, &node.node_data.name);
                                let mut updated_node = node.clone();
                                updated_node.node_data.name = new_endpoint.clone();

                                // Collect edges that need to be updated
                                let edges_to_update: Vec<_> = self
                                    .edges
                                    .iter()
                                    .filter(|((src, _), _)| src == key)
                                    .map(|((_, dst), edge)| (dst.clone(), edge.clone()))
                                    .collect();

                                updates.push((key.clone(), updated_node, edges_to_update));
                            }
                        }
                    }
                }
            }
        }

        // Apply all updates at once
        for (old_key, updated_node, edges) in updates {
            let new_key = create_node_key(updated_node.clone());

            // Update node
            self.nodes.remove(&old_key);
            self.nodes.insert(new_key.clone(), updated_node);

            // Update edges
            for (dst, edge) in edges {
                self.edges.remove(&(old_key.clone(), dst.clone()));
                self.edges.insert((new_key.clone(), dst), edge);
            }
        }

        Ok(())
    }
    fn class_includes(&mut self) {
        let class_nodes: Vec<_> = self
            .nodes
            .iter()
            .filter(|(_, n)| n.node_type == NodeType::Class)
            .map(|(k, n)| (k.clone(), n.clone()))
            .collect();

        for (_, node) in class_nodes {
            if let Some(includes) = node.node_data.meta.get("includes") {
                let modules = includes.split(',').map(|m| m.trim());
                for module in modules {
                    if let Some(module_node) =
                        self.find_nodes_by_name(NodeType::Class, module).first()
                    {
                        let edge = Edge::class_imports(&node.node_data, &module_node);
                        self.add_edge(edge);
                    }
                }
            }
        }
    }
    fn class_inherits(&mut self) {
        let class_nodes: Vec<_> = self
            .nodes
            .iter()
            .filter(|(_, n)| n.node_type == NodeType::Class)
            .map(|(k, n)| (k.clone(), n.clone()))
            .collect();

        for (_, node) in class_nodes {
            if let Some(parent) = node.node_data.meta.get("parent") {
                if let Some(parent_node) = self.find_nodes_by_name(NodeType::Class, parent).first()
                {
                    let edge = Edge::parent_of(&parent_node, &node.node_data);
                    self.add_edge(edge);
                }
            }
        }
    }
    fn get_data_models_within(&mut self, lang: &Lang) {
        let prefix = format!("{:?}-", NodeType::DataModel);

        let data_model_nodes: Vec<NodeData> = self
            .nodes
            .range(prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&prefix))
            .map(|(_, node)| node.node_data.clone())
            .collect();

        for data_model in data_model_nodes {
            let edges = lang.lang().data_model_within_finder(&data_model, &|file| {
                self.find_nodes_by_file_ends_with(NodeType::Function, file)
            });

            for edge in edges {
                self.add_edge(edge);
            }
        }
    }

    fn filter_out_nodes_without_children(
        &mut self,
        parent_type: NodeType,
        child_type: NodeType,
        child_meta_key: &str,
    ) {
        let mut has_children: BTreeMap<String, bool> = BTreeMap::new();

        let parent_prefix = format!("{:?}-", parent_type);
        for (_, node) in self
            .nodes
            .range(parent_prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&parent_prefix))
        {
            has_children.insert(node.node_data.name.clone(), false);
        }

        let child_prefix = format!("{:?}-", child_type);
        for (_, node) in self
            .nodes
            .range(child_prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&child_prefix))
        {
            if let Some(parent_name) = node.node_data.meta.get(child_meta_key) {
                if let Some(entry) = has_children.get_mut(parent_name) {
                    *entry = true;
                }
            }
        }

        let nodes_to_remove: Vec<_> = self
            .nodes
            .iter()
            .filter(|(_, node)| {
                node.node_type == parent_type
                    && !has_children.get(&node.node_data.name).unwrap_or(&true)
            })
            .map(|(k, _)| k.clone())
            .collect();

        for key in nodes_to_remove {
            self.nodes.remove(&key);
            // Remove associated edges
            self.edges
                .retain(|(src, dst), _| src != &key && dst != &key);
        }
    }

    fn prefix_paths(&mut self, root: &str) {
        let nodes_to_update: Vec<_> = self
            .nodes
            .iter()
            .map(|(k, node)| {
                let mut new_node = node.clone();
                new_node.add_root(root);
                (k.clone(), new_node)
            })
            .collect();

        for (key, node) in nodes_to_update {
            self.nodes.insert(key, node);
        }
    }

    fn find_nodes_by_name_contains(&self, node_type: NodeType, name: &str) -> Vec<NodeData> {
        let prefix = format!("{:?}-{}", node_type, name);
        self.nodes
            .range(prefix.clone()..)
            .take_while(|(k, n)| k.starts_with(&prefix) && n.node_data.name.contains(name))
            .map(|(_, node)| node.node_data.clone())
            .collect()
    }

    fn find_resource_nodes(&self, node_type: NodeType, verb: &str, path: &str) -> Vec<NodeData> {
        let prefix = format!("{:?}-", node_type);

        self.nodes
            .range(prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&prefix))
            .filter(|(_, node)| {
                let node_data = &node.node_data;

                // Check if path matches
                let path_matches = node_data.name.contains(path);

                // Check if verb matches (if present in metadata)
                let verb_matches = match node_data.meta.get("verb") {
                    Some(node_verb) => node_verb.to_uppercase() == verb.to_uppercase(),
                    None => true, // If no verb in metadata, don't filter on it
                };

                path_matches && verb_matches
            })
            .map(|(_, node)| node.node_data.clone())
            .collect()
    }

    fn find_handlers_for_endpoint(&self, endpoint: &NodeData) -> Vec<NodeData> {
        let endpoint = Node::new(NodeType::Endpoint, endpoint.clone());
        let endpoint_key = create_node_key(endpoint.clone());

        let mut handlers = Vec::new();

        for ((src, dst), edge_type) in &self.edges {
            if *edge_type == EdgeType::Handler && src == &endpoint_key {
                if let Some(node) = self.nodes.get(dst) {
                    handlers.push(node.node_data.clone());
                }
            }
        }

        handlers
    }

    fn check_direct_data_model_usage(&self, function_name: &str, data_model: &str) -> bool {
        for ((src_key, dst_key), edge_type) in &self.edges {
            if *edge_type == EdgeType::Contains {
                if let (Some(src_node), Some(dst_node)) =
                    (self.nodes.get(src_key), self.nodes.get(dst_key))
                {
                    if src_node.node_data.name == function_name
                        && dst_node.node_data.name.contains(data_model)
                    {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn find_functions_called_by(&self, function: &NodeData) -> Vec<NodeData> {
        let function_prefix = format!(
            "{:?}-{}-{}",
            NodeType::Function,
            function.name,
            function.file
        );
        let mut called_functions = Vec::new();

        for ((src, dst), edge_type) in &self.edges {
            if let EdgeType::Calls(_) = edge_type {
                if src.starts_with(&function_prefix) {
                    if let Some(node) = self.nodes.get(dst) {
                        called_functions.push(node.node_data.clone());
                    }
                }
            }
        }

        called_functions
    }

    fn find_nodes_by_type(&self, node_type: NodeType) -> Vec<NodeData> {
        let prefix = format!("{:?}-", node_type);
        self.nodes
            .range(prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&prefix))
            .map(|(_, node)| node.node_data.clone())
            .collect()
    }

    fn find_nodes_with_edge_type(
        &self,
        source_type: NodeType,
        target_type: NodeType,
        edge_type: EdgeType,
    ) -> Vec<(NodeData, NodeData)> {
        let mut result = Vec::new();
        let source_prefix = format!("{:?}-", source_type);
        let target_prefix = format!("{:?}-", target_type);

        for ((src_key, dst_key), edge) in &self.edges {
            if *edge == edge_type
                && src_key.starts_with(&source_prefix)
                && dst_key.starts_with(&target_prefix)
            {
                if let (Some(src_node), Some(dst_node)) =
                    (self.nodes.get(src_key), self.nodes.get(dst_key))
                {
                    result.push((src_node.node_data.clone(), dst_node.node_data.clone()));
                }
            }
        }

        result
    }
    fn count_edges_of_type(&self, edge_type: EdgeType) -> usize {
        self.edges
            .iter()
            .filter(|(_, edge)| match (edge, &edge_type) {
                (EdgeType::Calls(_), EdgeType::Calls(_)) => true,
                _ => **edge == edge_type,
            })
            .count()
    }
}

impl BTreeMapGraph {
    fn find_node_from_node_ref(&self, node_ref: NodeRef) -> Option<Node> {
        let prefix = format!(
            "{:?}-{}-{}",
            &node_ref.node_type, &node_ref.node_data.name, &node_ref.node_data.file
        );
        self.nodes
            .range(prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&prefix))
            .map(|(_, node)| node.clone())
            .next()
    }

    fn find_node_from_node_key(&self, node_type: NodeType, node_key: NodeKeys) -> Option<Node> {
        let prefix = format!("{:?}-{}-{}", &node_type, &node_key.name, &node_key.file);
        self.nodes
            .range(prefix.clone()..)
            .take_while(|(k, _)| k.starts_with(&prefix))
            .map(|(_, node)| node.clone())
            .next()
    }
}
impl Default for BTreeMapGraph {
    fn default() -> Self {
        BTreeMapGraph {
            nodes: BTreeMap::new(),
            edges: BTreeMap::new(),
        }
    }
}
