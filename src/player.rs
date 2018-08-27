use ::util::{Camera, Transform};
use ::input::InputState;
use winit::VirtualKeyCode;
use cgmath::{Point3, Vector3, Quaternion, Matrix4, Euler, Deg, InnerSpace};
use cgmath::Transform as CgmathTransform;


pub struct Player {
    pub position: Point3<f32>,
    pub camera: Camera,
    pub pitch: f64,
    pub yaw: f64,
    pub dimension_id: u32
}


impl Player {
    pub fn new() -> Player {
        Player {
            position: Point3::new(0.0, 0.0, 0.0),
            camera: Camera::new(),
            pitch: 0.0,
            yaw: 0.0,
            dimension_id: 0
        }
    }


    pub fn update(&mut self, dt: f64, input: &InputState) {
        if input.right_mouse_pressed {
            const MOUSE_SPEED: f64 = 3.0;
            self.yaw += input.mouse_delta.0 * MOUSE_SPEED * dt;
            self.pitch -= input.mouse_delta.1 * MOUSE_SPEED * dt;
            if self.pitch < -89.0 { self.pitch = -89.0; }
            if self.pitch > 89.0 { self.pitch = 89.0; }

            let mut move_vec = Vector3::new(0.0, 0.0, 0.0);
            if input.get_key_down(&VirtualKeyCode::W) { move_vec += Vector3::new(0.0, 0.0, -1.0); }
            if input.get_key_down(&VirtualKeyCode::S) { move_vec += Vector3::new(0.0, 0.0,  1.0); }
            if input.get_key_down(&VirtualKeyCode::A) { move_vec += Vector3::new(-1.0, 0.0, 0.0); }
            if input.get_key_down(&VirtualKeyCode::D) { move_vec += Vector3::new( 1.0, 0.0, 0.0); }
            move_vec = Matrix4::from_angle_y(Deg(-self.yaw as f32)).transform_vector(move_vec);
            if input.get_key_down(&VirtualKeyCode::Space) { move_vec += Vector3::new(0.0, 1.0, 0.0); }
            if input.get_key_down(&VirtualKeyCode::LControl) { move_vec += Vector3::new(0.0, -1.0, 0.0); }

            const MOVE_SPEED: f32 = 5.0;  // units per second
            let mut speed = MOVE_SPEED * dt as f32;
            if input.get_key_down(&VirtualKeyCode::LShift) { speed *= 3.0; }
            // can't normalize (0, 0, 0)
            if move_vec.magnitude() == 0.0 {
                move_vec = move_vec * speed;
            }
            else {
                move_vec = move_vec.normalize() * speed;
            }

            self.position += move_vec;
        }
    }


    pub fn get_transform(&self) -> Transform {
        Transform {
            position: self.position.clone(),
            rotation: Quaternion::from(Euler { x: Deg(-self.pitch as f32), y: Deg(self.yaw as f32), z: Deg(0f32) }),
            scale: 1.0
        }
    }
}
