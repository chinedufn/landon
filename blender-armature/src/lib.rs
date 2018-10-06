//! Blender files can have armature such as circles, cubes, cylinders, a dragon or any other
//! 3D shape.
//!
//! A armature can be represented as a group of vertices and data about those vertices, such as their
//! normals or UV coordinates.
//!
//! Armaturees can also have metadata, such as the name of it's parent armature (useful for vertex
//! skinning).
//!
//! blender-armature-to-json seeks to be a well tested, well documented exporter for blender armature
//! metadata.
//!
//! You can write data to stdout or to a file. At the onset it will be geared towards @chinedufn's
//! needs - but if you have needs that aren't met feel very free to open an issue.
//!
//! @see https://docs.blender.org/manual/en/dev/modeling/armature/introduction.html - Armature Introduction
//! @see https://github.com/chinedufn/blender-actions-to-json - Exporting blender armatures / actions

#[macro_use]
extern crate failure;
extern crate cgmath;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use cgmath::Matrix;
use cgmath::Matrix3;
use cgmath::Matrix4;
use cgmath::Quaternion;
use serde_json::Error;
use std::collections::HashMap;

mod interpolate;
pub use crate::interpolate::ActionSettings;
pub use crate::interpolate::InterpolationSettings;

/// Something went wrong in the Blender child process that was trying to parse your armature data.
#[derive(Debug, Fail)]
pub enum BlenderError {
    /// Errors in Blender are written to stderr. We capture the stderr from the `blender` child
    /// process that we spawned when attempting to export armature from a `.blend` file.
    #[fail(
        display = "There was an issue while exporting armature: Blender stderr output: {}",
        _0
    )]
    Stderr(String),
}

/// A bone in an armature. Can either be a dual quaternion or a matrix. When you export bones
/// from Blender they come as matrices - BlenderArmature lets you convert them into dual
/// quaternions which are usually more favorable for when implementing skeletal animation.
///
/// TODO: Maybe? Use cgmath::Matrix4 instead of our Vec<f32>. We'd want a custom serializer /
/// deserializer so that we don't need to litter our JSON with `Matrix4` object declarations
/// when we output it from Blender.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(Clone))]
pub enum Bone {
    Matrix(Vec<f32>),
    DualQuat(Vec<f32>),
}

/// All of the data about a Blender armature that we've exported from Blender.
/// A BlenderArmature should have all of the data that you need to implement skeletal
/// animation.
///
/// If you have other needs, such as a way to know the model space position of any bone at any
/// time so that you can, say, render a baseball in on top of your hand bone.. Open an issue.
/// (I plan to support this specific example in the future)
///
/// TODO: BlenderArmature.y_up() fixes the actions to be y up instead of z up
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(Default, Clone))]
pub struct BlenderArmature {
    pub joint_index: HashMap<String, u8>,
    pub inverse_bind_poses: Vec<Bone>,
    // TODO: Generic type instead of string for your action names so that you can have an enum
    // for your action names ... ?
    // TODO: Inner HashMap should have a float key not a string since it is a time in seconds
    // but you can't have floats as keys so need a workaround.
    // TODO: &str for action name instead of String?
    pub actions: HashMap<String, Vec<Keyframe>>,
}

/// The pose bones at an individual keyframe time
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(Default, Clone))]
pub struct Keyframe {
    frame_time_secs: f32,
    bones: Vec<Bone>,
}

impl BlenderArmature {
    /// Given a string of JSON we deserialize a BlenderArmature. This is here as a convenience
    /// since we already depend on serde anyway.
    /// In a real application you might want to serialize and deserialize to a smaller file
    /// format.. such as `bincode`.
    pub fn from_json(json_str: &str) -> Result<BlenderArmature, Error> {
        serde_json::from_str(json_str)
    }

    /// Convert a matrix into a dual quaternion
    /// https://github.com/chinedufn/mat4-to-dual-quat/blob/master/src/mat4-to-dual-quat.js
    /// Note that we use `w, x, y, z` and not `x, y, z, w` for our quaternion representation
    pub fn matrix_to_dual_quat(bone: &Bone) -> Bone {
        match bone {
            Bone::DualQuat(dual_quat) => Bone::DualQuat(dual_quat.to_vec()),
            Bone::Matrix(matrix) => {
                let mut cg_matrix_4 = BlenderArmature::matrix_array_to_slices(&matrix);
                let matrix4 = Matrix4::from(cg_matrix_4);

                // https://github.com/stackgl/gl-mat3/blob/master/from-mat4.js
                let mut mat3 = BlenderArmature::matrix4_to_mat3_array(matrix4);

                let rotation3 = Matrix3::from(mat3);
                let rotation_quat = Quaternion::from(rotation3);

                let mut trans_vec = vec![0.0];
                let mut t = matrix[12..15].to_vec();

                trans_vec.append(&mut t);

                let mut translation_vec = [0.0; 4];
                translation_vec.copy_from_slice(&trans_vec[..]);

                let trans_quat = Quaternion::from(translation_vec);
                let trans_quat = trans_quat * rotation_quat;
                let mut trans_quat = trans_quat * 0.5;

                let mut dual_quat: Vec<f32> = rotation_quat[0..4].to_vec();
                dual_quat.append(&mut trans_quat[..].to_vec());

                Bone::DualQuat(dual_quat)
            }
        }
    }

    /// https://github.com/chinedufn/dual-quat-to-mat4/blob/master/src/dual-quat-to-mat4.js
    pub fn dual_quat_to_matrix(bone: &Bone) -> Bone {
        match bone {
            Bone::Matrix(matrix) => Bone::Matrix(matrix.clone()),
            Bone::DualQuat(dual_quat) => {
                let mut matrix = vec![];
                matrix.resize(16, 0.0);
                let dq = dual_quat;

                matrix[0] = 1.0 - (2.0 * dq[2] * dq[2]) - (2.0 * dq[3] * dq[3]);
                matrix[1] = (2.0 * dq[1] * dq[2]) + (2.0 * dq[0] * dq[3]);
                matrix[2] = (2.0 * dq[1] * dq[3]) - (2.0 * dq[0] * dq[2]);
                matrix[3] = 0.0;
                matrix[4] = (2.0 * dq[1] * dq[2]) - (2.0 * dq[0] * dq[3]);
                matrix[5] = 1.0 - (2.0 * dq[1] * dq[1]) - (2.0 * dq[3] * dq[3]);
                matrix[6] = (2.0 * dq[2] * dq[3]) + (2.0 * dq[0] * dq[1]);
                matrix[7] = 0.0;
                matrix[8] = (2.0 * dq[1] * dq[3]) + (2.0 * dq[0] * dq[2]);
                matrix[9] = (2.0 * dq[2] * dq[3]) - (2.0 * dq[0] * dq[1]);
                matrix[10] = 1.0 - (2.0 * dq[1] * dq[1]) - (2.0 * dq[2] * dq[2]);
                matrix[11] = 0.0;
                matrix[12] = 2.0 * (-dq[4] * dq[1] + dq[5] * dq[0] - dq[6] * dq[3] + dq[7] * dq[2]);
                matrix[13] = 2.0 * (-dq[4] * dq[2] + dq[5] * dq[3] + dq[6] * dq[0] - dq[7] * dq[1]);
                matrix[14] = 2.0 * (-dq[4] * dq[3] - dq[5] * dq[2] + dq[6] * dq[1] + dq[7] * dq[0]);
                matrix[15] = 1.0;

                Bone::Matrix(matrix)
            }
        }
    }

    fn matrix_array_to_slices(matrix: &Vec<f32>) -> [[f32; 4]; 4] {
        let mut slices = [[0.0; 4]; 4];

        slices[0].copy_from_slice(&matrix[0..4]);
        slices[1].copy_from_slice(&matrix[4..8]);
        slices[2].copy_from_slice(&matrix[8..12]);
        slices[3].copy_from_slice(&matrix[12..16]);

        slices
    }

    fn matrix4_to_mat3_array(mat4: Matrix4<f32>) -> [[f32; 3]; 3] {
        // https://github.com/stackgl/gl-mat3/blob/master/from-mat4.js
        let mut mat3 = [[0.0; 3]; 3];
        let m = mat4;
        mat3[0].copy_from_slice(&[m[0][0], m[0][1], m[0][2]]);
        mat3[1].copy_from_slice(&[m[1][0], m[1][1], m[1][2]]);
        mat3[2].copy_from_slice(&[m[2][0], m[2][1], m[2][2]]);

        mat3
    }
}

// TODO: These methods can be abstracted into calling a method that takes a callback
impl BlenderArmature {
    /// Iterate over all of the action bones and apply and multiply in the inverse bind pose.
    ///
    /// TODO: another function to apply bind shape matrix? Most armatures seem to export an identity
    /// bind shape matrix but that might not be the same for every armature.
    pub fn apply_inverse_bind_poses(&mut self) {
        for (_name, action) in self.actions.iter_mut() {
            for keyframe in action.iter_mut() {
                for (index, bone) in keyframe.bones.iter_mut().enumerate() {
                    bone.multiply(&mut self.inverse_bind_poses[index]);
                }
            }
        }
    }

    /// Tranpose all of the bone matrices in our armature's action keyframes.
    /// Blender uses row major matrices, but OpenGL uses column major matrices so you'll
    /// usually want to transpose your matrices before using them.
    pub fn transpose_actions(&mut self) {
        for (_name, action) in self.actions.iter_mut() {
            for keyframe in action.iter_mut() {
                for (_index, bone) in keyframe.bones.iter_mut().enumerate() {
                    bone.transpose();
                }
            }
        }
    }
}

impl BlenderArmature {
    /// Convert your action matrices into dual quaternions so that you can implement
    /// dual quaternion linear blending.
    pub fn actions_to_dual_quats(&mut self) {
        for (_, keyframes) in self.actions.iter_mut() {
            for keyframe in keyframes.iter_mut() {
                for bone in keyframe.bones.iter_mut() {
                    *bone = BlenderArmature::matrix_to_dual_quat(bone);
                }
            }
        }
    }
}

impl Bone {
    fn multiply(&mut self, rhs: &mut Bone) {
        match self {
            Bone::Matrix(ref mut lhs_matrix) => match rhs {
                Bone::Matrix(ref mut rhs_matrix) => {
                    let lhs_slices = BlenderArmature::matrix_array_to_slices(lhs_matrix);
                    let lhs_mat4 = Matrix4::from(lhs_slices);

                    let rhs_slices = BlenderArmature::matrix_array_to_slices(rhs_matrix);
                    let rhs_mat4 = Matrix4::from(rhs_slices);

                    let multiplied = vec_from_matrix4(&(rhs_mat4 * lhs_mat4));

                    *lhs_matrix = multiplied;
                }
                Bone::DualQuat(_) => {}
            },
            Bone::DualQuat(_) => {}
        };
    }

    fn transpose(&mut self) {
        match self {
            Bone::Matrix(ref mut matrix) => {
                let slices = BlenderArmature::matrix_array_to_slices(matrix);
                let mut mat4 = Matrix4::from(slices);
                *matrix = vec_from_matrix4(&mat4.transpose());
            }
            Bone::DualQuat(_) => {}
        };
    }

    /// Get a vector representation of your bone data.
    /// You'll usually pass this vector of your bone data to the GPU.
    pub fn vec(&self) -> Vec<f32> {
        match self {
            Bone::Matrix(matrix) => matrix.clone(),
            Bone::DualQuat(dual_quat) => dual_quat.clone(),
        }
    }
}

pub type ArmatureNamesToData = HashMap<String, BlenderArmature>;
pub type FilenamesToArmaturees = HashMap<String, ArmatureNamesToData>;

/// Given a buffer of standard output from Blender we parse all of the armature JSON that was
/// written to stdout by `blender-armature-to-json.py`.
///
/// Armaturees data in stdout will look like:
///
/// START_ARMATURE_JSON /path/to/file.blend my_armature_name
/// {...}
/// END_ARMATURE_JSON /path/to/file.blend my_armature_name
///
/// @see blender-armature-to-json.py - This is where we write to stdout
pub fn parse_armatures_from_blender_stdout(
    blender_stdout: &str,
) -> Result<FilenamesToArmaturees, failure::Error> {
    let mut filenames_to_armature = HashMap::new();

    let mut index = 0;

    while let Some((filename_to_armature, next_start_index)) =
        find_first_armature_after_index(blender_stdout, index)
    {
        filenames_to_armature.extend(filename_to_armature);
        index = next_start_index;
    }

    Ok(filenames_to_armature)
}

fn find_first_armature_after_index(
    blender_stdout: &str,
    index: usize,
) -> Option<(FilenamesToArmaturees, usize)> {
    let blender_stdout = &blender_stdout[index as usize..];

    if let Some(armature_start_index) = blender_stdout.find("START_ARMATURE_JSON") {
        let mut filenames_to_armature = HashMap::new();
        let mut armature_name_to_data = HashMap::new();

        let armature_end_index = blender_stdout.find("END_ARMATURE_JSON").unwrap();

        let armature_data = &blender_stdout[armature_start_index..armature_end_index];

        let mut lines = armature_data.lines();

        let first_line = lines.next().unwrap();

        let armature_filename: Vec<&str> = first_line.split(" ").collect();
        let armature_filename = armature_filename[1].to_string();

        let armature_name = first_line.split(" ").last().unwrap().to_string();

        let armature_data: String = lines.collect();
        let armature_data: BlenderArmature = serde_json::from_str(&armature_data).expect(&format!(
            "Could not deserialize Blender Armature data{}",
            &armature_data
        ));

        armature_name_to_data.insert(armature_name, armature_data);
        filenames_to_armature.insert(armature_filename, armature_name_to_data);

        return Some((filenames_to_armature, armature_end_index + 1));
    }

    return None;
}

fn vec_from_matrix4(mat4: &Matrix4<f32>) -> Vec<f32> {
    // TODO: Accept output vec instead of re-allocating
    let mut vec = vec![];

    for index in 0..16 {
        vec.push(mat4[index / 4][index % 4]);
    }

    vec
}

#[cfg(test)]
mod tests {
    use super::*;

    // Concatenate a series of vectors into one vector
    macro_rules! concat_vecs {
        ( $( $vec:expr),* ) => {
            {
                let mut concatenated_vec = Vec::new();
                $(
                    concatenated_vec.append(&mut $vec.clone());
                )*
                concatenated_vec
            }
        }
    }

    // TODO: dual_quat_z_up_to_y_up... but we can just get the rendering working first
    // https://github.com/chinedufn/change-mat4-coordinate-system/blob/master/change-mat4-coordinate-system.js

    #[test]
    fn matrix_to_dual_quat_and_back_again() {
        struct MatrixToDualQuatTest {
            matrix: Vec<f32>,
            dual_quat: Vec<f32>,
        }

        let tests = vec![
            MatrixToDualQuatTest {
                matrix: vec![
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
                ],
                dual_quat: vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            },
            MatrixToDualQuatTest {
                matrix: concat_vecs!(
                    vec![-0.8488113, -0.52869576, 0.00018605776, 0.0],
                    vec![0.52503425, -0.8428914, 0.117783956, 0.0],
                    vec![-0.06211505, 0.100074045, 0.99303925, 0.0],
                    vec![0.09010744, -0.23331697, 0.018946884, 1.0]
                ),
                dual_quat: concat_vecs!(
                    vec![-0.2744706, -0.01613097, 0.056746617, 0.9597841],
                    vec![-0.0017457254, -0.124870464, -0.011375335, -0.00192535]
                ),
            },
        ];

        for test in tests {
            let MatrixToDualQuatTest { matrix, dual_quat } = test;
            let round = 10_000.0;

            let matrix_bone = Bone::Matrix(matrix.clone());
            let dual_quat_bone = Bone::DualQuat(dual_quat.clone());

            if let Bone::Matrix(new_matrix) = BlenderArmature::dual_quat_to_matrix(&dual_quat_bone)
            {
                // Round values to remove precision errors
                let new_matrix: Vec<f32> = new_matrix.iter().map(|x| x * round / round).collect();
                let matrix: Vec<f32> = matrix.iter().map(|x| x * round / round).collect();
                assert_eq!(new_matrix, matrix);
            } else {
                panic!();
            }

            if let Bone::DualQuat(new_dual_quat) =
                BlenderArmature::matrix_to_dual_quat(&matrix_bone)
            {
                let new_dual_quat: Vec<f32> =
                    new_dual_quat.iter().map(|x| (x * round).round()).collect();
                let dual_quat: Vec<f32> = dual_quat.iter().map(|x| (x * round).round()).collect();
                assert_eq!(new_dual_quat, dual_quat);
            } else {
                panic!();
            }
        }
    }

    #[test]
    fn applying_inv_bind_poses() {
        let mut start_actions = HashMap::new();
        let mut keyframes = vec![];
        keyframes.push(Keyframe {
            frame_time_secs: 1.0,
            bones: vec![Bone::Matrix(concat_vecs!(
                vec![1.0, 6.0, 2.0, 1.0],
                vec![7.0, 1.0, 2.0, 5.0],
                vec![0.0, 4.0, 1.0, 0.0],
                vec![0.0, 0.0, 0.0, 1.0]
            ))],
        });
        start_actions.insert("Fly".to_string(), keyframes);

        let mut start_armature = BlenderArmature {
            actions: start_actions,
            inverse_bind_poses: vec![Bone::Matrix(concat_vecs!(
                vec![1.0, 0.0, 0.0, 0.0],
                vec![0.0, 1.0, 0.0, 0.0],
                vec![0.0, 0.0, 1.0, 0.0],
                vec![0.0, 0.0, 5.0, 1.0]
            ))],
            ..BlenderArmature::default()
        };

        start_armature.apply_inverse_bind_poses();

        let mut end_actions = HashMap::new();
        let mut keyframes = vec![];
        keyframes.push(Keyframe {
            frame_time_secs: 1.0,
            bones: vec![Bone::Matrix(concat_vecs!(
                vec![1.0, 6.0, 7.0, 1.0],
                vec![7.0, 1.0, 27.0, 5.0],
                vec![0.0, 4.0, 1.0, 0.0],
                vec![0.0, 0.0, 5.0, 1.0]
            ))],
        });
        end_actions.insert("Fly".to_string(), keyframes);

        let expected_armature = BlenderArmature {
            actions: end_actions,
            ..start_armature.clone()
        };

        assert_eq!(start_armature, expected_armature);
    }

    #[test]
    fn convert_actions_to_dual_quats() {
        let mut start_actions = HashMap::new();
        let mut keyframes = vec![];
        keyframes.push(Keyframe {
            frame_time_secs: 1.0,
            bones: vec![Bone::Matrix(concat_vecs!(
                vec![1.0, 0.0, 0.0, 0.0],
                vec![0.0, 1.0, 0.0, 0.0],
                vec![0.0, 0.0, 1.0, 0.0],
                vec![0.0, 0.0, 0.0, 1.0]
            ))],
        });
        start_actions.insert("Fly".to_string(), keyframes);

        let mut start_armature = BlenderArmature {
            actions: start_actions,
            ..BlenderArmature::default()
        };

        start_armature.actions_to_dual_quats();

        let mut end_actions = HashMap::new();
        let mut keyframes = vec![];
        keyframes.push(Keyframe {
            frame_time_secs: 1.0,
            bones: vec![Bone::DualQuat(concat_vecs!(
                vec![1.0, 0.0, 0.0, 0.0],
                vec![0.0, 0.0, 0.0, 0.0]
            ))],
        });
        end_actions.insert("Fly".to_string(), keyframes);

        let expected_armature = BlenderArmature {
            actions: end_actions,
            ..start_armature.clone()
        };

        assert_eq!(start_armature, expected_armature);
    }

    // TODO: Function to return these start_actions that we keep using
    #[test]
    fn transpose_actions() {
        let mut start_actions = HashMap::new();
        let mut keyframes = vec![];
        keyframes.push(Keyframe {
            frame_time_secs: 1.0,
            bones: vec![Bone::Matrix(concat_vecs!(
                vec![1.0, 0.0, 0.0, 0.0],
                vec![0.0, 1.0, 0.0, 0.0],
                vec![0.0, 0.0, 1.0, 0.0],
                vec![0.0, 0.0, 5.0, 1.0]
            ))],
        });

        start_actions.insert("Fly".to_string(), keyframes);

        let mut start_armature = BlenderArmature {
            actions: start_actions,
            ..BlenderArmature::default()
        };

        start_armature.transpose_actions();

        let mut end_actions = HashMap::new();
        let mut keyframes = vec![];
        keyframes.push(Keyframe {
            frame_time_secs: 1.0,
            bones: vec![Bone::Matrix(concat_vecs!(
                vec![1.0, 0.0, 0.0, 0.0],
                vec![0.0, 1.0, 0.0, 0.0],
                vec![0.0, 0.0, 1.0, 5.0],
                vec![0.0, 0.0, 0.0, 1.0]
            ))],
        });
        end_actions.insert("Fly".to_string(), keyframes);

        let expected_armature = BlenderArmature {
            actions: end_actions,
            ..start_armature.clone()
        };

        assert_eq!(start_armature, expected_armature);
    }
}
