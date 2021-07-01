use super::vertex::Vertex;
use std::fmt;
pub struct VertexPack {
    pub vertices: Vec<Vertex>,
    pub elements: Vec<u32>,
}
impl VertexPack {
    ///TODO: Make this follow the contract
    pub fn new(vertices: Vec<Vertex>, elements: Option<Vec<u32>>) -> VertexPack {
        let elements = match elements {
            Some(e) => e,
            None => Vec::new(),
        };
        VertexPack { vertices, elements }
    }
}

impl fmt::Debug for VertexPack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VertexPack")
    }
}
