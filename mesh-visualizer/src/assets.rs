//! Managers the loading and storage of our assets.
//! Namely, meshes and armatures that came from Blender and textures png's.

use bincode;
use blender_armature::BlenderArmature;
use blender_mesh::BlenderMesh;
use js_sys::Promise;
use js_sys::Uint8Array;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::convert::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::*;

type Meshes = Rc<RefCell<HashMap<String, BlenderMesh>>>;
type Armatures = Rc<RefCell<HashMap<String, BlenderArmature>>>;

#[wasm_bindgen]
extern "C" {
    /// Bridge the gap for things that are currently unsupported or difficult to accomplish with
    /// wasm-bindgen
    pub type WasmHelpers;

    #[wasm_bindgen(static_method_of = WasmHelpers)]
    pub fn fetch_u8_array(url: &str, callback: &js_sys::Function);
}

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

    pub fn load_meshes(&mut self) {
        let request_url = "/dist/meshes.bytes";

        let meshes_clone = Rc::clone(&self.meshes);

        let deserialize_meshes = move |mesh_bytes: Box<[u8]>| {
            let meshes: HashMap<String, BlenderMesh> = bincode::deserialize(&mesh_bytes).unwrap();

            for (mesh_name, mut mesh) in meshes {
                web_sys::console::log_1(&format!("{}", mesh_name).into());
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

        let closure = Closure::wrap(Box::new(deserialize_meshes) as Box<FnMut(_)>);

        let callback = closure.as_ref().unchecked_ref();
        WasmHelpers::fetch_u8_array("/dist/meshes.bytes", callback);

        closure.forget();
    }

    // TODO: Temporarily commented out while I refactor
    pub fn load_armature(&mut self, armature_name: &str) {
        //        let armatures_clone = Rc::clone(&self.armatures);
        //
        //        let deserialize_armatures = move |armatures_json: &[u8]| {
        //            let armatures: HashMap<String, BlenderArmature> =
        //                bincode::deserialize(&armatures_json).unwrap();
        //
        //            for (armature_name, mut armature) in armatures {
        //                armature.apply_inverse_bind_poses();
        //                armature.transpose_actions();
        //                armature.actions_to_dual_quats();
        //
        //                armatures_clone
        //                    .borrow_mut()
        //                    .insert(armature_name.to_string(), armature);
        //            }
        //        };
        //
        //        let on_armatures_downloaded = Closure::new(deserialize_armatures);
        //
        //        let _model_path = &format!("dist/{}.json", armature_name);
        //        download_string("/dist/armatures.json".to_string(), &on_armatures_downloaded);
        //
        //        on_armatures_downloaded.forget();
    }

    pub fn meshes(&self) -> Meshes {
        Rc::clone(&self.meshes)
    }

    pub fn armatures(&self) -> Armatures {
        Rc::clone(&self.armatures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foo() {}
}
