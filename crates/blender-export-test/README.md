# Blender Exporter Test

> Tests against real `.blend` files that verify that our exporter is works correctly in different scenarios.

## Meshes

### ./src/principled_material_no_input_nodes.blend

A cube with a principled material node that has no nodes feeding into it.

We're verifying that we properly read the values from the sliders that are on the Principled BSDF node.

### ./src/principled_material_uniform_input_nodes.blend

A cube with a principled material node with other uniform nodes feeding into it.

We're verifying that we properly read the values from those nodes.

## ./src/principled_material_texture_inputs.blend

A cube with a principled material node with texture nodes feeding into it.

We're verifying that we properly export the names of the textures.

### ./src/suzanne.blend

A high poly Suzanne mesh. We don't run any tests against it - but we do render it in the mesh-visualizer.

Useful for visualizing things that are more easily seen with more geometry - such as lighting.
