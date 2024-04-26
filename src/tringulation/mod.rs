use vertex::Vertex;
use crate::gen::Chunk;
pub mod vertex;
struct Tringle(Vertex, Vertex, Vertex);

pub struct Mesh {
    tringles: Vec<Tringle>,
}

impl Mesh {
    pub fn tringulate_chunk(chunk: &Chunk) -> Self {
        Self {
            tringles: Vec::new(),
        }
    }
}

