use godot::classes::*;
use godot::prelude::*;

use std::collections::BTreeMap;

#[derive(GodotClass)]
#[class(no_init, base=Node)]
pub struct ObjectPool {
    scene: Gd<PackedScene>,
    alive: BTreeMap<InstanceId, Gd<Node3D>>,
    dead: Vec<Gd<Node3D>>,
    base: Base<Node>,
}

/*
// TODO: do we need to create a PoolableObject to have a better cleanup?
#[godot_api]
impl ObjectPool {
    #[signal]
    fn on_killed(object: Gd<Node3D>);
}
*/

impl ObjectPool {
    fn instantiate_new(scene: &Gd<PackedScene>, index: usize) -> Gd<Node3D> {
        let mut instance = scene.try_instantiate_as::<Node3D>().unwrap();

        let name = instance.get_name();
        instance.set_name(&format!("{name}_{index}"));

        instance.set_physics_process(true);
        instance.set_process(true);
        instance.hide();

        godot_print!("instantiating object: {}", instance.get_name());

        instance
    }

    pub fn create(scene: Gd<PackedScene>) -> Gd<Self> {
        let mut pool = Gd::from_init_fn(|base| {
            let mut dead = Vec::with_capacity(8);
            for i in 0..dead.capacity() {
                let instance = Self::instantiate_new(&scene, i);
                dead.push(instance);
            }

            let alive = BTreeMap::new();

            Self {
                scene,
                alive,
                dead,
                base,
            }
        });

        let mut pool_iter_clone = pool.clone();
        for obj in pool_iter_clone.bind_mut().dead.iter_mut() {
            /*
            // XXX: is there a way to make this not use the low-level Godot signal handling?
            let on_object_killed = pool.callable("on_object_killed");
            obj.connect("on_killed", &on_object_killed);
            */

            pool.add_child(&obj.clone());
        }

        pool
    }

    pub fn get_or_instantiate(&mut self) -> Gd<Node3D> {
        let mut instance = self.dead.pop().unwrap_or_else(|| {
            let index = self.alive.len() + self.dead.len();
            let instance = Self::instantiate_new(&self.scene, index);
            self.base_mut().add_child(&instance);
            instance
        });

        self.alive.insert(instance.instance_id(), instance.clone());

        instance.set_physics_process(true);
        instance.set_process(true);
        instance.show();

        instance
    }

    pub fn return_all_to_pool(&mut self) {
        godot_print!("kill all");

        // Refactor: this is allocating more memory
        let alive_objects: Vec<_> = self.alive.values().cloned().collect();
        alive_objects
            .into_iter()
            .for_each(|obj| self.return_to_pool(obj));

        self.alive.clear();
    }

    pub fn return_to_pool<T: Inherits<Node3D>>(&mut self, object: Gd<T>) {
        let mut object = object.upcast();

        godot_print!(
            "returning object to pool: {} -> {}",
            object.get_name(),
            self.to_gd().get_name()
        );

        object.set_physics_process(true);
        object.set_process(true);
        object.hide();

        self.alive.remove(&object.instance_id());

        object.reparent(&self.to_gd());
        object.set_position(Vector3::ZERO);
        object.set_rotation_degrees(Vector3::ZERO);

        self.dead.push(object);
    }

    /*
    // TODO: do we need to create a PoolableObject to have a better cleanup?
    fn on_object_killed(&mut self, object: Gd<Node3D>) {
        self.return_to_pool(object);
    }
    */
}
