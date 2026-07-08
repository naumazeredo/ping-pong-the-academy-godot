use godot::classes::*;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub struct GridSystem {
    #[export]
    camera: Option<Gd<Camera3D>>,

    // Ground plane used to raycast from camera to position structures in the layers
    #[init(val = Plane::new(Vector3::UP, 0.0))]
    ground_plane: Plane,

    cache_frame: i32,
    mouse_projection: Option<Vector3>,

    base: Base<Node3D>,
}

impl GridSystem {
    fn calculate_mouse_projection(&mut self) {
        let mouse_position = self.base().get_viewport().unwrap().get_mouse_position();
        let view_camera = self.camera.as_ref().unwrap();

        self.mouse_projection = self.ground_plane.intersect_ray(
            view_camera.project_ray_origin(mouse_position),
            view_camera.project_ray_normal(mouse_position),
        );
    }

    pub fn get_mouse_projection(&mut self) -> Option<Vector3> {
        let current_frame = Engine::singleton().get_frames_drawn();
        if self.cache_frame != current_frame {
            self.cache_frame = current_frame;
            self.calculate_mouse_projection();
        }

        self.mouse_projection
    }

    pub fn get_grid_cell(&self, mouse_projection: Option<Vector3>) -> Option<Vector2i> {
        mouse_projection.map(|Vector3 { x, y: _, z }| {
            Vector2i::new(x.as_f32().floor() as i32, z.as_f32().floor() as i32)
        })
    }

    pub fn get_grid_corner(&self, mouse_projection: Option<Vector3>) -> Option<Vector2i> {
        mouse_projection.map(|Vector3 { x, y: _, z }| {
            Vector2i::new(
                (x.as_f32() + 0.5).floor() as i32,
                (z.as_f32() + 0.5).floor() as i32,
            )
        })
    }
}
