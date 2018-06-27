//! Managers the loading and storage of our assets.
//! Namely, meshes and armatures that came from Blender and textures png's.

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use blender_mesh::BlenderMesh;
use download_model;
use wasm_bindgen::prelude::*;

struct Assets {
    /// All of our Blender models that w have downloaded and can render
    meshes: Rc<RefCell<HashMap<String, BlenderMesh>>>,
}

impl Assets {
        fn load_model(&mut self, model_name: &str) {
        let meshes_clone = Rc::clone(&self.meshes);

        let save_model_in_state = move |model_json: String| {
            let mut mesh = BlenderMesh::from_json(&model_json).unwrap();

            mesh.combine_vertex_indices();
            mesh.triangulate();
            mesh.y_up();

            meshes_clone
                .borrow_mut()
                .insert("dist/LetterF.json".to_string(), mesh);
        };

        let on_model_load = Closure::new(save_model_in_state);

        download_model("dist/LetterF.json", &on_model_load);

        // TODO: Instead of calling .forget() and leaking memory every time we load a model,
        // see if can store it our
        // struct as an option and re-use the closure / only forget it once
        on_model_load.forget();
    }
}
