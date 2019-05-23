use crate::assets::Assets;
use crate::shader::Shader;
use crate::shader::ShaderSystem;
use crate::shader::ShaderType;
use crate::state_wrapper::{State, StateWrapper};
use blender_armature::ActionSettings;
use blender_armature::BlenderArmature;
use blender_armature::Bone;
use blender_armature::InterpolationSettings;
use blender_mesh::BlenderMesh;
use js_sys::WebAssembly;
use nalgebra::Perspective3;
use nalgebra::{Isometry3, Matrix4, Point3, Vector3};
use std::cell::RefCell;
use std::f32::consts::PI;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::WebGlRenderingContext as GL;
use web_sys::*;

mod armature_render;
mod mesh_render;

pub struct Renderer {
    gl: Rc<WebGlRenderingContext>,
    assets: Rc<RefCell<Assets>>,
    shader_sys: Rc<ShaderSystem>,
}

trait Render {
    fn shader_type(&self) -> ShaderType;
    fn render(&self, gl: &WebGlRenderingContext, shader_program: &Shader);
    // FIXME: Better paradigm.. Only buffer once and use VAO..
    fn buffer_f32_data(
        &self,
        gl: &WebGlRenderingContext,
        buf: Option<&WebGlBuffer>,
        // TODO: &Vec<f32>
        data: &Vec<f32>,
        attrib_loc: i32,
        size: u8,
    ) {
        gl.bind_buffer(GL::ARRAY_BUFFER, buf);

        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();

        let data_location = data.as_ptr() as u32 / 4;

        let data_array = js_sys::Float32Array::new(&memory_buffer)
            .subarray(data_location, data_location + data.len() as u32);

        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &data_array, GL::STATIC_DRAW);

        // TODO: buffer_u8_data and use gl_byte for joint indices
        gl.vertex_attrib_pointer_with_i32(attrib_loc as u32, size as i32, GL::FLOAT, false, 0, 0);
    }
}
trait BlenderMeshRender {
    fn render_non_skinned(&self, gl: &WebGlRenderingContext, shader_program: &Shader);
    fn render_dual_quat_skinned(&self, gl: &WebGlRenderingContext, shader_program: &Shader);
}

struct attribute<T>(T);
struct uniform<T>(T);

// TODO: These types can probably be automatically generated based on the shader
struct NonSkinnedRender {
    aVertexPos: attribute<Vec<f32>>,
    aVertexNormal: attribute<Vec<f32>>,
    uMVMatrix: uniform<Vec<f32>>,
}

impl Render for BlenderMesh {
    fn shader_type(&self) -> ShaderType {
        if let Some(_) = self.armature_name {
            ShaderType::DualQuatSkin
        } else {
            ShaderType::NonSkinned
        }
    }
    fn render(&self, gl: &WebGlRenderingContext, shader: &Shader) {
        if let Some(_) = self.armature_name {
            self.render_dual_quat_skinned(&gl, &shader);
        } else {
            self.render_non_skinned(&gl, &shader);
        }
    }
}

impl BlenderMeshRender for BlenderMesh {
    fn render_non_skinned(&self, gl: &WebGlRenderingContext, shader: &Shader) {
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();

        let vertex_pos_attrib =
            gl.get_attrib_location(shader.program.as_ref().unwrap(), "aVertexPosition");
        gl.enable_vertex_attrib_array(vertex_pos_attrib as u32);

        let vertex_normal_attrib =
            gl.get_attrib_location(shader.program.as_ref().unwrap(), "aVertexNormal");
        gl.enable_vertex_attrib_array(vertex_normal_attrib as u32);

        let fovy = PI / 3.0;
        let perspective = Perspective3::new(fovy, 1.0, 0.1, 50.0);

        let mut perspective_array = [0.; 16];
        perspective_array.copy_from_slice(perspective.as_matrix().as_slice());

        let perspective_uni =
            gl.get_uniform_location(shader.program.as_ref().unwrap(), "perspective");
        let perspective_uni = perspective_uni.as_ref();
        gl.uniform_matrix4fv_with_f32_array(perspective_uni, false, &mut perspective_array);

        // TODO: state.camera
        let eye = Point3::new(1.0, 8.0, 10.0);
        let target = Point3::new(0.0, 0.0, 0.0);
        let view = Isometry3::look_at_rh(&eye, &target, &Vector3::y());

        let view = view.to_homogeneous();

        let pos = (0.0, 0.0, 0.0);
        let model = Isometry3::new(Vector3::new(pos.0, pos.1, pos.2), nalgebra::zero());
        let model = model.to_homogeneous();

        let mut model_array = [0.; 16];
        let mut view_array = [0.; 16];

        model_array.copy_from_slice(model.as_slice());
        view_array.copy_from_slice(view.as_slice());

        let model_uni = gl.get_uniform_location(shader.program.as_ref().unwrap(), "model");
        let model_uni = model_uni.as_ref();

        let view_uni = gl.get_uniform_location(shader.program.as_ref().unwrap(), "view");
        let view_uni = view_uni.as_ref();

        gl.uniform_matrix4fv_with_f32_array(model_uni, false, &mut model_array);
        gl.uniform_matrix4fv_with_f32_array(view_uni, false, &mut view_array);

        // TODO: Breadcrumb - add normal and point lighting to shader..

        // TODO: Multiply without new allocation

        let pos = &self.vertex_positions;
        self.buffer_f32_data(&gl, shader.buffers[0].as_ref(), pos, vertex_pos_attrib, 3);

        let norms = &self.vertex_normals;
        self.buffer_f32_data(
            &gl,
            shader.buffers[1].as_ref(),
            norms,
            vertex_normal_attrib,
            3,
        );

        if let Some(ref uvs) = self.vertex_uvs.as_ref() {
            let texture_coord_attrib =
                gl.get_attrib_location(shader.program.as_ref().unwrap(), "aTextureCoord");
            gl.enable_vertex_attrib_array(texture_coord_attrib as u32);

            self.buffer_f32_data(
                &gl,
                shader.buffers[2].as_ref(),
                uvs,
                texture_coord_attrib,
                2,
            );
        }

        let indices = &self.vertex_position_indices;

        let indices_location = indices.as_ptr() as u32 / 2;
        let indices_array = js_sys::Uint16Array::new(&memory_buffer)
            .subarray(indices_location, indices_location + indices.len() as u32);

        let index_buffer = gl.create_buffer().unwrap();
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
        gl.buffer_data_with_array_buffer_view(
            GL::ELEMENT_ARRAY_BUFFER,
            &indices_array,
            GL::STATIC_DRAW,
        );

        gl.draw_elements_with_i32(GL::TRIANGLES, indices.len() as i32, GL::UNSIGNED_SHORT, 0);
    }

    fn render_dual_quat_skinned(&self, gl: &WebGlRenderingContext, shader: &Shader) {
        //        let vertex_pos_attrib = gl.get_attrib_location(&shader.program, "aVertexPosition");
        //        gl.enable_vertex_attrib_array(vertex_pos_attrib);
        //
        //        let vertex_normal_attrib = gl.get_attrib_location(&shader.program, "aVertexNormal");
        //        gl.enable_vertex_attrib_array(vertex_normal_attrib);
        //
        //        let joint_index_attrib = gl.get_attrib_location(&shader.program, "aJointIndex");
        //        gl.enable_vertex_attrib_array(joint_index_attrib);
        //
        //        let joint_weight_attrib = gl.get_attrib_location(&shader.program, "aJointWeight");
        //        gl.enable_vertex_attrib_array(joint_weight_attrib);
        //
        //        let texture_coord_attrib = gl.get_attrib_location(&shader.program, "aTextureCoord");
        //        gl.enable_vertex_attrib_array(texture_coord_attrib);
        //
        //        let fovy = cgmath::Rad(PI / 3.0);
        //        let perspective = cgmath::perspective(fovy, 1.0, 0.1, 100.0);
        //        let p_matrix = vec_from_matrix4(&perspective);
        //
        //        let model_matrix = Matrix4::from_translation(Vector3::new(0.0, 0.0, 0.0));
        //
        //        let mut mv_matrix = Matrix4::look_at(
        //            Point3::new(1.0, 2.0, 2.0),
        //            Point3::new(0.0, 0.0, 0.0),
        //            Vector3::new(0.0, 1.0, 0.0),
        //        );
        //
        //        // TODO: Breadcrumb - add normal and point lighting to shader..
        //
        //        // TODO: Multiply without new allocation
        //        mv_matrix = mv_matrix * model_matrix;
        //
        //        let mv_matrix = vec_from_matrix4(&mv_matrix);
        //
        //        let p_matrix_uni = gl.get_uniform_location(&shader.program, "uPMatrix");
        //        gl.uniform_matrix_4fv(p_matrix_uni, false, p_matrix);
        //
        //        let mv_matrix_uni = gl.get_uniform_location(&shader.program, "uMVMatrix");
        //        gl.uniform_matrix_4fv(mv_matrix_uni, false, mv_matrix);
        //
        //        let pos = self.vertex_positions.clone();
        //        self.buffer_f32_data(&gl, &shader.buffers[0], pos, vertex_pos_attrib, 3);
        //
        //        let norms = self.vertex_normals.clone();
        //        self.buffer_f32_data(&gl, &shader.buffers[1], norms, vertex_normal_attrib, 3);
        //
        //        let joints = vec_u8_to_f32(self.vertex_group_indices.as_ref().unwrap().clone());
        //        self.buffer_f32_data(&gl, &shader.buffers[2], joints, joint_index_attrib, 4);
        //
        //        let weights = self.vertex_group_weights.as_ref().unwrap().clone();
        //        self.buffer_f32_data(&gl, &shader.buffers[3], weights, joint_weight_attrib, 4);
        //
        //        let uvs = self.vertex_uvs.as_ref().unwrap().clone();
        //        self.buffer_f32_data(&gl, &shader.buffers[4], uvs, texture_coord_attrib, 2);
        //
        //        let index_buffer = gl.create_buffer();
        //        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, &index_buffer);
        //
        //        // TODO: Remove clone
        //        let pos_idx = self.vertex_position_indices.clone();
        //        gl.buffer_u16_data(GL::ELEMENT_ARRAY_BUFFER, pos_idx, GL::STATIC_DRAW);
        //
        //        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, &index_buffer);
        //
        //        let pos_idx_len = self.vertex_position_indices.len();
        //        gl.draw_elements(GL::TRIANGLES, pos_idx_len as u16, GL::UNSIGNED_SHORT, 0);
    }
}

impl Renderer {
    pub fn new(
        gl: Rc<WebGlRenderingContext>,
        assets: Rc<RefCell<Assets>>,
        shader_sys: Rc<ShaderSystem>,
    ) -> Renderer {
        Renderer {
            gl,
            assets,
            shader_sys,
        }
    }

    pub fn render(&self, state: &State) {
        self.gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        let mesh = self.assets.borrow().meshes();
        let mesh = mesh.borrow();
        let mesh = mesh.get(state.current_model.as_str());

        if mesh.is_none() {
            return;
        }

        let mesh = mesh.unwrap();

        self.shader_sys.use_program(&mesh.shader_type());

        let shader = self.shader_sys.get_shader(&mesh.shader_type());

        if mesh.armature_name.is_some() {
            let armature = self.assets.borrow().armatures();
            let armature = armature.borrow();
            let armature = armature.get(mesh.armature_name.as_ref().unwrap());

            if armature.is_none() {
                return;
            }

            armature
                .unwrap()
                .buffer_data(&self.gl, shader.unwrap(), &state);
        }

        mesh.render(&self.gl, shader.unwrap());
    }
}

trait ArmatureDataBuffer {
    fn buffer_data(&self, gl: &WebGlRenderingContext, shader: &Shader, state: &State);
}

impl ArmatureDataBuffer for BlenderArmature {
    fn buffer_data(&self, gl: &WebGlRenderingContext, shader: &Shader, state: &State) {
        //        let now = State::performance_now_to_system_time();
        //
        //        let current_time = now.duration_since(state.app_start_time).unwrap();
        //        let seconds = current_time.as_secs();
        //        let millis = current_time.subsec_millis();
        //        let current_time_secs = seconds as f32 + (millis as f32 / 1000.0);
        //
        //        let interp_opts = InterpolationSettings {
        //            current_time: current_time_secs,
        //            // TODO: self.get_bone_group(BlenderArmature::ALL_BONES)
        //            joint_indices: vec![0, 1, 2, 3],
        //            blend_fn: None,
        //
        //            current_action: ActionSettings::new("Twist", 0.0, true),
        //            previous_action: None,
        //        };
        //        let bones = self.interpolate_bones(&interp_opts);
        //
        //        let mut bones: Vec<(&u8, &Bone)> = bones.iter().to_owned().collect();
        //        bones.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap());
        //        let bones: Vec<&Bone> = bones.iter().map(|(_, bone)| *bone).collect();
        //
        //        for (index, bone) in bones.iter().enumerate() {
        //            let bone = bone.vec();
        //            let (rot_quat, trans_quat) = bone.split_at(4);
        //
        //            let rot_quat = rot_quat.to_vec();
        //            let rot_quat_uni = &format!("boneRotQuaternions[{}]", index);
        //            let rot_quat_uni = gl.get_uniform_location(&shader.program, rot_quat_uni);
        //            gl.uniform_4fv(rot_quat_uni, rot_quat);
        //
        //            let trans_quat = trans_quat.to_vec();
        //            let trans_quat_uni = &format!("boneTransQuaternions[{}]", index);
        //            let trans_quat_uni = gl.get_uniform_location(&shader.program, trans_quat_uni);
        //            gl.uniform_4fv(trans_quat_uni, trans_quat);
        //        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hi() {}
}
