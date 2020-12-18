//! Functions for converting between matrix and dual quaternion bones

use crate::BlenderArmature;
use crate::Bone;
use nalgebra::{DualQuaternion, Matrix4, Quaternion};

impl BlenderArmature {
    /// Convert a matrix into a dual quaternion
    /// https://github.com/chinedufn/mat4-to-dual-quat/blob/master/src/mat4-to-dual-quat.js
    pub fn matrix_to_dual_quat(bone: &Bone) -> Bone {
        match bone {
            Bone::DualQuat(_dual_quat) => panic!("Already a dual quaternion"),
            Bone::Matrix(matrix) => {
                let matrix = matrix.as_slice();

                let matrix3 = [
                    matrix[0], matrix[1], matrix[2], matrix[4], matrix[5], matrix[6], matrix[8],
                    matrix[9], matrix[10],
                ];

                // i, j, k, w
                let rotation_quat = quaternion_from_mat3(matrix3);

                // w, i, j, k
                let rotation_quat = Quaternion::new(
                    rotation_quat[3],
                    rotation_quat[0],
                    rotation_quat[1],
                    rotation_quat[2],
                );

                // w, i, j, k
                let trans_quat = Quaternion::new(0.0, matrix[12], matrix[13], matrix[14]);
                let trans_quat = trans_quat * rotation_quat;
                let trans_quat = trans_quat * 0.5;

                Bone::DualQuat(DualQuaternion::new(rotation_quat, trans_quat))
            }
        }
    }

    /// https://github.com/chinedufn/dual-quat-to-mat4/blob/master/src/dual-quat-to-mat4.js
    pub fn dual_quat_to_matrix(bone: &Bone) -> Bone {
        match bone {
            Bone::Matrix(matrix) => Bone::Matrix(matrix.clone()),
            Bone::DualQuat(dual_quat) => {
                let mut matrix: [f32; 16] = [0.0; 16];

                let dq = [
                    dual_quat.rot.w,
                    dual_quat.rot.i,
                    dual_quat.rot.j,
                    dual_quat.rot.k,
                    dual_quat.trans.w,
                    dual_quat.trans.i,
                    dual_quat.trans.j,
                    dual_quat.trans.k,
                ];

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

                Bone::Matrix(Matrix4::from_column_slice(&matrix))
            }
        }
    }
}

// https://github.com/stackgl/gl-quat/blob/master/fromMat3.js
// [i, j, k, w]
fn quaternion_from_mat3(m: [f32; 9]) -> [f32; 4] {
    // Algorithm in Ken Shoemake's article in 1987 SIGGRAPH course notes
    // article "Quaternion Calculus and Fast Animation".
    let f_trace = m[0] + m[4] + m[8];

    let mut out = [0.0; 4];

    if f_trace > 0.0 {
        let mut f_root = (f_trace + 1.0).sqrt();
        out[3] = 0.5 * f_root;
        f_root = 0.5 / f_root;
        out[0] = (m[5] - m[7]) * f_root;
        out[1] = (m[6] - m[2]) * f_root;
        out[2] = (m[1] - m[3]) * f_root;
    } else {
        let mut i = 0;
        if m[4] > m[0] {
            i = 1;
        }
        if m[8] > m[i * 3 + i] {
            i = 2;
        }
        let j = (i + 1) % 3;
        let k = (i + 2) % 3;

        let mut f_root = (m[i * 3 + i] - m[j * 3 + j] - m[k * 3 + k] + 1.0).sqrt();
        out[i] = 0.5 * f_root;
        f_root = 0.5 / f_root;
        out[3] = (m[j * 3 + k] - m[k * 3 + j]) * f_root;
        out[j] = (m[j * 3 + i] + m[i * 3 + j]) * f_root;
        out[k] = (m[k * 3 + i] + m[i * 3 + k]) * f_root;
    }

    return out;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpolate::tests::dq_to_bone;

    #[test]
    fn matrix_to_dual_quat_and_back_again() {
        struct MatrixToDualQuatTest {
            matrix: [f32; 16],
            dual_quat: [f32; 8],
        }

        let tests = vec![
            MatrixToDualQuatTest {
                matrix: [
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
                ],
                dual_quat: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            },
            MatrixToDualQuatTest {
                matrix: [
                    //
                    -0.8488113,
                    -0.52869576,
                    0.00018605776,
                    0.0,
                    //
                    0.52503425,
                    -0.8428914,
                    0.117783956,
                    0.0,
                    //
                    -0.06211505,
                    0.100074045,
                    0.99303925,
                    0.0,
                    //
                    0.09010744,
                    -0.23331697,
                    0.018946884,
                    1.0,
                ],
                dual_quat: [
                    //
                    -0.2744706,
                    -0.01613097,
                    0.056746617,
                    0.9597841,
                    //
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

            let matrix_bone = Bone::Matrix(Matrix4::from_column_slice(&matrix));
            let dual_quat_bone = dq_to_bone(dual_quat);

            if let Bone::Matrix(new_matrix) = BlenderArmature::dual_quat_to_matrix(&dual_quat_bone)
            {
                let new_matrix = new_matrix.as_slice();

                // Round values to remove precision errors
                let new_matrix: Vec<f32> = new_matrix.iter().map(|x| x * round / round).collect();
                let matrix: Vec<f32> = matrix.iter().map(|x| x * round / round).collect();
                assert_eq!(new_matrix, matrix);
            } else {
                unreachable!();
            }

            if let Bone::DualQuat(new_dual_quat) =
                BlenderArmature::matrix_to_dual_quat(&matrix_bone)
            {
                let new_dual_quat = [
                    new_dual_quat.rot.w,
                    new_dual_quat.rot.i,
                    new_dual_quat.rot.j,
                    new_dual_quat.rot.k,
                    new_dual_quat.trans.w,
                    new_dual_quat.trans.i,
                    new_dual_quat.trans.j,
                    new_dual_quat.trans.k,
                ];

                let new_dual_quat: Vec<f32> =
                    new_dual_quat.iter().map(|x| (x * round).round()).collect();
                let dual_quat: Vec<f32> = dual_quat.iter().map(|x| (x * round).round()).collect();
                assert_eq!(new_dual_quat, dual_quat);
            } else {
                unreachable!();
            }
        }
    }
}
