use glam::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(8.0, 4.0, 8.0),
            yaw: 0.0,
            pitch: 0.0,
            distance: 3.0,
        }
    }

    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch = (self.pitch + delta_pitch).clamp(-1.54, 1.54); // ~+-88 degrees
    }

    pub fn move_forward(&mut self) {
        let right = Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin());
        self.position += right * 0.1;
    }

    pub fn move_backward(&mut self) {
        let right = Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin());
        self.position -= right * 0.1;
    }

    pub fn move_left(&mut self) {
        let forward = Vec3::new(self.yaw.sin(), 0.0, -self.yaw.cos());
        self.position += forward * 0.1;
    }

    pub fn move_right(&mut self) {
        let forward = Vec3::new(self.yaw.sin(), 0.0, -self.yaw.cos());
        self.position -= forward * 0.1;
    }

    pub fn fly_up(&mut self) {
        self.position.y += 0.1;
    }

    pub fn fly_down(&mut self) {
        self.position.y -= 0.1;
    }

    pub fn create_view_proj(&self, aspect: f32) -> [[f32; 4]; 4] {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let forward = Vec3::new(cy * cp, sp, sy * cp);
        let eye = self.position;
        let target = self.position + forward;
        let up = Vec3::Y;
        let view = Mat4::look_at_rh(eye, target, up);
        let proj = Mat4::perspective_rh_gl(45.0_f32.to_radians(), aspect, 0.1, 100.0);
        (proj * view).to_cols_array_2d()
    }

    pub fn view_proj_mat(&self, aspect: f32) -> Mat4 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let forward = Vec3::new(cy * cp, sp, sy * cp);
        let eye = self.position;
        let target = self.position + forward;
        let up = Vec3::Y;
        let view = Mat4::look_at_rh(eye, target, up);
        let proj = Mat4::perspective_rh_gl(45.0_f32.to_radians(), aspect, 0.1, 100.0);
        proj * view
    }
} 