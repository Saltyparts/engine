use cgmath::{
    Deg,
    Matrix4,
    perspective,
    Point3,
    Quaternion,
    Vector3,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub position: Point3<f32>,
    pub rotation: Quaternion<f32>,
    pub fov: f32,
    pub near_plane: f32,
    pub far_plane: f32,
}

impl Camera {
    pub fn new(position: Point3<f32>, rotation: Quaternion<f32>, fov: f32, near_plane: f32, far_plane: f32) -> Camera {
        Camera {
            position,
            rotation,
            fov,
            near_plane,
            far_plane,
        }
    }

    pub fn view_matrix(&self, aspect_ratio: f32) -> [f32; 16] {
        let mx_projection = perspective(Deg(self.fov), aspect_ratio, self.near_plane, self.far_plane);
        let mx_view = Matrix4::look_to_rh(self.position, self.rotation * Vector3::unit_z(), Vector3::unit_y());
        //let mx_view: Matrix4<f32> = self.rotation.into();
        // this is required to convert opengl matrices to wgpu matrices
        let mx_correction = cgmath::Matrix4::new(
            1., 0., 0. , 0.,
            0., 1., 0. , 0.,
            0., 0., 0.5, 0.,
            0., 0., 0.5, 1.,
        );

        let matrix: Matrix4<f32> = mx_correction * mx_projection * mx_view;
        let matrix: [[f32; 4]; 4] = matrix.into();
        bytemuck::cast(matrix)
    }
}
