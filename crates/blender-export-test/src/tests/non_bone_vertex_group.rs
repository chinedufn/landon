use crate::filesystem::blender_export_test_fixtures;
use crate::tests::test_utils::export_meshes_from_blender_file;

/// We were previously assuming that all vertex groups were bone groups.
///
/// This test verifies that we can export skinned meshes that have vertex groups
/// that are not armature bones.
#[test]
fn export_mesh_with_non_bone_group() {
    let meshes = export_meshes_from_blender_file(
        blender_export_test_fixtures().join("non_bone_vertex_group.blend"),
    );
    assert!(meshes.get("Cube").is_some());
}
