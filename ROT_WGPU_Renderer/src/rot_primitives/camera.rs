use nalgebra as na;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: na::Matrix4<f32> = na::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new(camera: &Camera) -> Self {
        let mut view_proj: [[f32; 4]; 4] = [[0.0; 4]; 4];
        for (index, elem) in camera.build_view_projection_matrix().iter().enumerate() {
            let collumn = index % 4;
            let line = index / 16;

            view_proj[collumn][line] = *elem;
        }

        Self { view_proj }
    }
}

pub struct Camera {
    pub eye: na::Point3<f32>,
    pub target: na::Point3<f32>,
    pub up: na::Vector3<f32>,

    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> na::Matrix4<f32> {
        let view = na::Matrix4::face_towards(&self.eye, &self.target, &self.up);
        let proj = na::Matrix4::new_perspective(self.aspect, self.fovy, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * view * proj;
    }
}
