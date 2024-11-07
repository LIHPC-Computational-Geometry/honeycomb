use honeycomb_core::cmap::VertexIdentifier;

pub struct NodeTri {
    v: VertexIdentifier,
    neighbors: [VertexIdentifier; 2],
}

pub struct Node {
    v: VertexIdentifier,
    neighbors: Vec<VertexIdentifier>,
}
