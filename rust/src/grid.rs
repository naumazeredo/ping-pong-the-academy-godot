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

    mouse_projection: Option<Vector3>,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for GridSystem {
    fn process(&mut self, _delta: f64) {
        self.calculate_mouse_projection();
    }
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

    pub fn get_mouse_projection(&self) -> Option<Vector3> {
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
