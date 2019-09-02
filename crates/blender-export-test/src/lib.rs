pub mod basic_cube;
pub mod multiple_armatures;
pub mod multiple_meshes;
pub mod principled_material_no_input_nodes;
pub mod principled_material_normal_map;
pub mod principled_material_single_channel_input;
pub mod principled_material_texture_inputs;
pub mod principled_material_uniform_input_nodes;
pub mod skinned_letter_f;
pub mod textured_cube;

mod filesystem;

fn set_active_object_by_name(name: &str) -> String {
    format!(
        r#"
import bpy
bpy.context.view_layer.objects.active = None
for obj in bpy.context.scene.objects:
    if obj.name == '{}':
        bpy.context.view_layer.objects.active = obj
"#,
        name
    )
}
