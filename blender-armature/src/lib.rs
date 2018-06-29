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

// TODO: - breadcrumb -> convert this file into blender armature.. and add armature export
// to letter_f.rs test and verify that it matches the expected BlenderArmature

#[macro_use]
extern crate failure;
extern crate cgmath;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use cgmath::Matrix3;
use cgmath::Quaternion;
use serde_json::Error;
use std::cmp::max;
use std::collections::HashMap;
use std::collections::HashSet;

/// Something went wrong in the Blender child process that was trying to parse your armature data.
#[derive(Debug, Fail)]
pub enum BlenderError {
    /// Errors in Blender are written to stderr. We capture the stderr from the `blender` child
    /// process that we spawned when attempting to export armature from a `.blend` file.
    #[fail(display = "There was an issue while exporting armature: Blender stderr output: {}", _0)]
    Stderr(String),
}

/// TODO: Use cgmath::Matrix4 instead of our own custom matrix. We'll want a custom serializer /
/// deserializer so that we don't need to litter our JSON with the names of our Rust structs
/// when we output it from Blender.
///
/// But for now it's fine to litter out JSON while we get things working..
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Bone {
    Matrix(Vec<f32>),
    DualQuat(Vec<f32>),
}

/// All of the data about a Blender armature
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(test, derive(Default))]
pub struct BlenderArmature {
    pub joint_index: HashMap<String, u8>,
    pub inverse_bind_poses: Vec<Bone>,
    // TODO: Generic type instead of string for your action names so that you can have an enum
    // for your action names ... ?
    pub actions: HashMap<String, HashMap<String, Vec<Bone>>>,
}

impl BlenderArmature {
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
                let mut cg_matrix_4 = [[0.0; 4]; 4];

                cg_matrix_4[0].copy_from_slice(&matrix[0..4]);
                cg_matrix_4[1].copy_from_slice(&matrix[4..8]);
                cg_matrix_4[2].copy_from_slice(&matrix[8..12]);
                cg_matrix_4[3].copy_from_slice(&matrix[12..16]);

                let matrix4 = cgmath::Matrix4::from(cg_matrix_4);

                // https://github.com/stackgl/gl-mat3/blob/master/from-mat4.js
                let mut mat3 = [[0.0; 3]; 3];
                let m = matrix4;
                mat3[0].copy_from_slice(&[m[0][0], m[0][1], m[0][2]]);
                mat3[1].copy_from_slice(&[m[1][0], m[1][1], m[1][2]]);
                mat3[2].copy_from_slice(&[m[2][0], m[2][1], m[2][2]]);

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
        let armature_data: BlenderArmature = serde_json::from_str(&armature_data).unwrap();

        armature_name_to_data.insert(armature_name, armature_data);
        filenames_to_armature.insert(armature_filename, armature_name_to_data);

        return Some((filenames_to_armature, armature_end_index + 1));
    }

    return None;
}

#[cfg(test)]
mod tests {
    use super::*;

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
                matrix: vec![
                    -0.8488113,
                    -0.52869576,
                    0.00018605776,
                    0.0,
                    0.52503425,
                    -0.8428914,
                    0.117783956,
                    0.0,
                    -0.06211505,
                    0.100074045,
                    0.99303925,
                    0.0,
                    0.09010744,
                    -0.23331697,
                    0.018946884,
                    1.0,
                ],
                dual_quat: vec![
                    -0.2744706,
                    -0.01613097,
                    0.056746617,
                    0.9597841,
                    -0.0017457254,
                    -0.124870464,
                    -0.011375335,
                    -0.00192535,
                ],
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
                BlenderArmature::matrix_to_dual_quat(&dual_quat_bone)
            {
                let new_dual_quat: Vec<f32> =
                    new_dual_quat.iter().map(|x| x * round / round).collect();
                let dual_quat: Vec<f32> = dual_quat.iter().map(|x| x * round / round).collect();
                assert_eq!(new_dual_quat, dual_quat);
            } else {
                panic!();
            }
        }
    }
}
