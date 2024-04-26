use cgmath::Vector3;

struct Voxel {
    // rockiness: u8, // maybe much later
    // type: u8, // maybe later
    exists: bool, // ahh better
}

impl Voxel {
    // fn rockiness(&self) -> f32 {
    //     self.rockiness as f32 / 255.0
    // }
}

const CHUNK_WIDTH: usize = 32;
const CHUNK_DEPTH: usize = 32;
const CHUNK_HEIGHT: usize = 32;

pub struct Chunk {
    blocks: [Voxel; CHUNK_WIDTH * CHUNK_DEPTH * CHUNK_HEIGHT],
    pos: Vector3<u32>,
}

impl Chunk {
    // pub fn generate(/*noise*/pos: Vector3<u32>) -> Self {
    //     Self {
    //         blocks: [],
    //         pos
    //     }
    // }
}
