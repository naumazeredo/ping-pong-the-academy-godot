use super::*;

use godot::classes::*;
use godot::prelude::*;

use std::collections::BTreeMap;

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct ObjectPools {
    pools: BTreeMap<String, Gd<ObjectPool>>,
    base: Base<Node>,
}

impl ObjectPools {
    pub fn get_or_create_pool(&mut self, scene: Gd<PackedScene>) -> Gd<ObjectPool> {
        let mut self_gd = self.to_gd();
        self.pools
            .entry(scene.get_path().to_string())
            .or_insert_with(|| {
                let pool = ObjectPool::create(scene.clone());
                self_gd.add_child(&pool);
                pool
            })
            .clone()
    }
}

#[derive(GodotClass)]
#[class(no_init, base=Node)]
pub struct ObjectPool {
    scene: Gd<PackedScene>,
    dead: Vec<Gd<StructureInstance>>,
    count: u32,
    base: Base<Node>,
}

/*
// TODO: do we need to create a PoolableObject to have a better cleanup?
#[godot_api]
impl ObjectPool {
    #[signal]
    fn on_killed(object: Gd<PlacedStructure>);
}
*/

impl ObjectPool {
    pub fn create(scene: Gd<PackedScene>) -> Gd<Self> {
        let mut pool = Gd::from_init_fn(|base| Self {
            scene,
            dead: Vec::new(),
            count: 0,
            base,
        });

        let mut dead = Vec::with_capacity(8);
        for _ in 0..dead.capacity() as u32 {
            let instance = pool.bind_mut().instantiate_new();
            dead.push(instance);
        }

        pool.bind_mut().dead = dead;

        let dead_0_name = pool.bind().dead[0].get_name();
        pool.set_name(&format!("Pool-{}", dead_0_name.trim_suffix("_0")));

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

    fn instantiate_new(&mut self) -> Gd<StructureInstance> {
        let mut instance = self
            .scene
            .try_instantiate_as::<StructureInstance>()
            .unwrap();

        let name = instance.get_name();
        instance.set_name(&format!("{name}_{}", self.count));

        instance.set_physics_process(true);
        instance.set_process(true);
        instance.hide();

        instance.bind_mut().assign_pool(self.to_gd());

        godot_print!("instantiating object: {}", instance.get_name());

        self.count += 1;
        instance
    }

    pub fn get_or_instantiate(&mut self) -> Gd<StructureInstance> {
        let mut instance = if self.dead.is_empty() {
            let instance = self.instantiate_new();
            self.base_mut().add_child(&instance);
            instance
        } else {
            let mut instance = self.dead.pop().unwrap();
            instance.bind_mut().unset_fields();
            instance
        };

        instance.set_physics_process(true);
        instance.set_process(true);
        instance.show();

        instance
    }

    pub fn return_to_pool<T: Inherits<StructureInstance>>(&mut self, object: Gd<T>) {
        let mut object = object.upcast();

        godot_print!(
            "returning object to pool: {} -> {}",
            object.get_name(),
            self.to_gd().get_name()
        );

        object.set_physics_process(true);
        object.set_process(true);
        object.hide();

        object.reparent(&self.to_gd());
        object.set_rotation_degrees(Vector3::ZERO);

        self.dead.push(object);
    }

    /*
    // TODO: do we need to create a PlacedStructure to have a better cleanup?
    fn on_object_killed(&mut self, object: Gd<PlacedStructure>) {
        self.return_to_pool(object);
    }
    */
}
