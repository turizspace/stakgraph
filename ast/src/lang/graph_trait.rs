use crate::lang::{Edge, Node, NodeType};

use super::{
    asg::{NodeData, NodeKeys},
    graph::EdgeType,
};

pub trait Graph {
    fn find_nodes_by_name(&self, node_type: NodeType, name: &str) -> Vec<NodeData>;
    fn find_nodes_in_range(&self, node_type: NodeType, row: u32, file: &str) -> Option<NodeData>;
    fn find_node_at(&self, node_type: NodeType, file: &str, line: u32) -> Option<NodeData>;
    fn find_node_by_name_in_file(
        &self,
        node_type: NodeType,
        name: &str,
        file: &str,
    ) -> Option<NodeData>;
    fn find_node_by_name_and_file_end_with(
        &self,
        node_type: NodeType,
        name: &str,
        suffix: &str,
    ) -> Option<NodeData>;
    fn find_nodes_by_file_ends_with(&self, node_type: NodeType, file: &str) -> Vec<NodeData>;
    // this method is used only in ruby (so far)

    fn find_source_edge_by_name_and_file(
        &self,
        edge_type: EdgeType,
        target_name: &str,
        target_file: &str,
    ) -> Option<NodeKeys>;

    // fn extend_node(&mut self, node: Node, parent_file: Option<&str>);
    // fn extend_edge(&mut self, edge: Edge, parent_file: Option<&str>);
}
