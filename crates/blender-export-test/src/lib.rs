#[cfg(test)]
mod tests;

#[cfg(test)]
mod filesystem;

#[cfg(test)]
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
