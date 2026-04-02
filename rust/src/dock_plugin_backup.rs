use godot::classes::editor_dock::{DockLayout, DockSlot};
use godot::classes::{Button, EditorDock, EditorPlugin, HBoxContainer, IEditorPlugin};

// -------------------------
// HOW to do the dock window
// -------------------------

#[derive(GodotClass)]
#[class(tool, init, base=EditorPlugin)]
struct MyEditorPlugin {
    dock: Option<Gd<EditorDock>>,
    button: Option<Gd<Button>>,
    hbox_container: Option<Gd<HBoxContainer>>,

    base: Base<EditorPlugin>,
}

#[godot_api]
impl IEditorPlugin for MyEditorPlugin {
    fn enter_tree(&mut self) {
        let mut dock = EditorDock::new_alloc();

        dock.set_title("My Dock");
        dock.set_default_slot(DockSlot::LEFT_UL);
        dock.set_available_layouts(DockLayout::ALL);

        let mut hbox_container = HBoxContainer::new_alloc();

        let mut button = Button::new_alloc();
        button.set_text("Test button");

        hbox_container.add_child(&button.clone().upcast::<Node>());
        dock.add_child(&hbox_container.clone().upcast::<Node>());

        self.base_mut().add_dock(Some(&dock));
        self.dock = Some(dock);
        self.button = Some(button);
        self.hbox_container = Some(hbox_container);
    }

    fn exit_tree(&mut self) {
        let mut button = self.button.take().unwrap();
        button.queue_free();

        let mut hbox_container = self.hbox_container.take().unwrap();
        hbox_container.queue_free();

        let dock = self.dock.take();
        self.base_mut().remove_dock(dock.as_ref());
        dock.unwrap().queue_free();
    }
}

// -------------------------------------
// HOW to do the button in the inspector
// -------------------------------------

#[derive(GodotClass)]
#[class(tool, init, base=Node3D)]
struct BuildingLayer {
    #[export_tool_button(fn = Self::on_meshlib_generate, name = "Generate MeshLibrary")]
    generate_meshlib_button: PhantomVar<Callable>,

    base: Base<Node3D>,
}

impl BuildingLayer {
    fn on_meshlib_generate(&mut self) {
        //let mut meshlib = MeshLibrary::new_gd();

        godot_print!("meshlib generate");
    }
}
