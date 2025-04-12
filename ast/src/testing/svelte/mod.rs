use crate::lang::graphs::NodeType;
use crate::lang::Graph;
use crate::{lang::Lang, repo::Repo};
use std::str::FromStr;
use test_log::test;

pub async fn test_svelte_generic<G: Graph>() -> Result<(), anyhow::Error> {
    let repo = Repo::new(
        "src/testing/svelte",
        Lang::from_str("svelte").unwrap(),
        false,
        Vec::new(),
        Vec::new(),
    )
    .unwrap();

    let graph = repo.build_graph_inner::<G>().await?;

    let (num_nodes, num_edges) = graph.get_graph_size();
    assert_eq!(num_nodes, 35, "Expected 35 nodes");
    assert_eq!(num_edges, 35, "Expected 35 edges");

    let language_nodes = graph.find_nodes_by_type(NodeType::Language);
    assert_eq!(language_nodes.len(), 1, "Expected 1 language node");
    assert_eq!(
        language_nodes[0].name, "svelte",
        "Language node name should be 'svelte'"
    );
    assert_eq!(
        language_nodes[0].file, "src/testing/svelte/",
        "Language node file path is incorrect"
    );

    let files = graph.find_nodes_by_type(NodeType::File);
    assert_eq!(files.len(), 6, "Expected 6 files");

    let imports = graph.find_nodes_by_type(NodeType::Import);
    assert_eq!(imports.len(), 1, "Expected 1 imports");

    let classes = graph.find_nodes_by_type(NodeType::Class);
    assert_eq!(classes.len(), 1, "Expected 1 classes");
    assert_eq!(classes[0].name, "Person", "Class name should be 'Person'");

    let functions = graph.find_nodes_by_type(NodeType::Function);
    assert_eq!(functions.len(), 15, "Expected 15 functions");

    let mut sorted_functions = functions.clone();
    sorted_functions.sort_by(|a, b| a.name.cmp(&b.name));

    assert_eq!(
        functions.iter().any(|f| f.name == "addPerson"),
        true,
        "Expected 'addPerson' function not found"
    );

    let data_models = graph.find_nodes_by_type(NodeType::DataModel);
    assert_eq!(data_models.len(), 1, "Expected 1 data models");

    let requests = graph.find_nodes_by_type(NodeType::Request);
    assert_eq!(requests.len(), 5, "Expected 5 request");
    assert_eq!(
        requests[0].name, "fetch",
        "Request name should be 'fetch'"
    );

    Ok(())
}

#[test(tokio::test)]
async fn test_svelte() {
    use crate::lang::graphs::ArrayGraph;
    test_svelte_generic::<ArrayGraph>().await.unwrap();
}

// #[test(tokio::test)]
// async fn test_svelte_btree() {
//     use crate::lang::graphs::BTreeMapGraph;
//     test_svelte_generic::<BTreeMapGraph>().await.unwrap();
// }