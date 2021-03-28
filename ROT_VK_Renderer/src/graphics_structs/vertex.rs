#[derive(Default, Copy, Clone)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

#[derive(Default, Copy, Clone)]
pub struct Vertex2D {
    pub position: [f32; 2],
    pub color: [f32; 4],
}
vulkano::impl_vertex!(Vertex2D, position, color);
