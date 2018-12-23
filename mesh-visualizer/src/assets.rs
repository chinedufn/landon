//! Managers the loading and storage of our assets.
//! Namely, meshes and armatures that came from Blender and textures png's.

use crate::download_texture;
use blender_armature::BlenderArmature;
use blender_mesh::BlenderMesh;
use serde_json;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

type Meshes = Rc<RefCell<HashMap<String, BlenderMesh>>>;
type Armatures = Rc<RefCell<HashMap<String, BlenderArmature>>>;
use crate::download_string;

pub struct Assets {
    /// All of our Blender models that we have downloaded and can render
    meshes: Meshes,
    /// All of our Blender armatures that we have downloaded and can render
    armatures: Armatures,
}

impl Assets {
    pub fn new() -> Assets {
        Assets {
            meshes: Rc::new(RefCell::new(HashMap::new())),
            armatures: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn load_mesh(&mut self, mesh_name: &str) {
        let meshes_clone = Rc::clone(&self.meshes);

        let deserialize_meshes = move |meshes_json: String| {
            let meshes: HashMap<String, BlenderMesh> = serde_json::from_str(&meshes_json).unwrap();

            for (mesh_name, mut mesh) in meshes {
                mesh.combine_vertex_indices();
                mesh.triangulate();
                mesh.y_up();

                if let Some(_) = mesh.armature_name {
                    mesh.set_groups_per_vertex(4);
                }

                meshes_clone
                    .borrow_mut()
                    .insert(mesh_name.to_string(), mesh);
            }
        };

        let on_meshes_downloaded = Closure::new(deserialize_meshes);

        let _model_path = &format!("dist/{}.json", mesh_name);
        download_string("/dist/meshes.json".to_string(), &on_meshes_downloaded);

        // TODO: Instead of calling .forget() and leaking memory every time we load a model,
        // see if can store it our
        // struct as an option and re-use the closure / only forget it once
        on_meshes_downloaded.forget();
    }

    // TODO: Temporarily commented out while I refactor
    pub fn load_armature(&mut self, armature_name: &str) {
        let armatures_clone = Rc::clone(&self.armatures);

        let deserialize_armatures = move |armatures_json: String| {
            let armatures: HashMap<String, BlenderArmature> =
                serde_json::from_str(&armatures_json).unwrap();

            for (armature_name, mut armature) in armatures {
                armature.apply_inverse_bind_poses();
                armature.transpose_actions();
                armature.actions_to_dual_quats();

                armatures_clone
                    .borrow_mut()
                    .insert(armature_name.to_string(), armature);
            }
        };

        let on_armatures_downloaded = Closure::new(deserialize_armatures);

        let _model_path = &format!("dist/{}.json", armature_name);
        download_string("/dist/armatures.json".to_string(), &on_armatures_downloaded);

        on_armatures_downloaded.forget();
    }

    pub fn meshes(&self) -> Meshes {
        Rc::clone(&self.meshes)
    }

    pub fn armatures(&self) -> Armatures {
        Rc::clone(&self.armatures)
    }
}
