//! Managers the loading and storage of our assets.
//! Namely, meshes and armatures that came from Blender and textures png's.

use blender_mesh::BlenderMesh;
use download_mesh;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

type Meshes = Rc<RefCell<HashMap<String, BlenderMesh>>>;

pub struct Assets {
    /// All of our Blender models that w have downloaded and can render
    meshes: Meshes,
}

impl Assets {
    pub fn new() -> Assets {
        Assets {
            meshes: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn load_mesh(&mut self, mesh_name: &str) {
        let meshes_clone = Rc::clone(&self.meshes);

        let save_model_in_state = move |mesh_name: String, mesh_json: String| {
            let mut mesh = BlenderMesh::from_json(&mesh_json).unwrap();

            mesh.combine_vertex_indices();
            mesh.triangulate();
            mesh.y_up();

            meshes_clone
                .borrow_mut()
                .insert(mesh_name.to_string(), mesh);
        };

        let on_model_load = Closure::new(save_model_in_state);

        let model_path = &format!("dist/{}.json", mesh_name);
        download_mesh(mesh_name, model_path, &on_model_load);

        // TODO: Instead of calling .forget() and leaking memory every time we load a model,
        // see if can store it our
        // struct as an option and re-use the closure / only forget it once
        on_model_load.forget();
    }

    pub fn meshes(&self) -> Meshes {
        Rc::clone(&self.meshes)
    }
}
