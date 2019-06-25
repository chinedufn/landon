# Blender Exporter Test

> Tests against real `.blend` files that verify that our exporter is works correctly in different scenarios.

## Meshes

### ./src/principled_material_no_input_nodes.blend

A cube with a principled material node that has no nodes feeding into it.

We're verifying that we properly read the values from the sliders that are on the Principled BSDF node.

### ./src/principled_material_uniform_input_nodes.blend

A cube with a principled material node with other uniform nodes feeding into it.

We're verifying that we properly read the values from those nodes.

### ./src/principled_material_texture_inputs.blend

A cube with a principled material node with texture nodes feeding into it.

We're verifying that we properly export the names of the textures.

### ./src/suzanne.blend

A high poly Suzanne mesh. We don't run any tests against it - but we do render it in the mesh-visualizer.

Useful for visualizing things that are more easily seen with more geometry - such as lighting.

### ./src/principled_material_single_channel_input.blend

A mesh with a principled material node that has a `Separate RGB` node feeding into the roughness (R channel)
and metallic (G channel).

We're verifying that we properly export information on single channel input data.

### ./src/principled_material_normal_map.blend

We're verifying that we can export the name of the normal map for the mesh.

## Images

### ./src/1x1-green-pixel.png

Used when we need any image and are not particularly concerned about its contents.

Example - when verifying that exporting a mesh also exports the name of one of its associated images.

We just care about the name, not the contents.
