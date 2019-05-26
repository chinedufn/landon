# The goal of this addon is to export all of the actions for the active
# armature into a JSON file

bl_info = {
    "name": "Export Mesh to JSON",
    "category": "Import-Export"
}

import bpy
import json
import os
from mathutils import Vector

# Write our JSON to stdout by default or to a file if specified.
# Stdout mesh JSON is wrapped in a start and end indicators
# to more easily distinguish it from other Blender output.
#
# START_MESH_JSON $BLENDER_FILEPATH $MESH_NAME
# ... mesh json ...
# END_MESH_JSON $BLENDER_FILEPATH $MESH_NAME
class MeshToJSON(bpy.types.Operator):
    """Given an active armature, export it's actions and keyframed bone pose information to a JSON file"""
    # Unique identifier for the addon
    bl_idname = 'import_export.mesh2json'
    # Display name in the interface
    bl_label = 'Export Mesh to JSON'
    bl_options = {'REGISTER'}
    bl_category = 'Import-Export'

    # The filepath to write out JSON to
    # filepath = bpy.props.StringProperty(name='filepath')

    def execute(self, context):
        bpy.ops.object.mode_set(mode = 'OBJECT')

        mesh = bpy.context.active_object
        mesh_json = {
            'vertex_positions': [],
            'num_vertices_in_each_face': [],
            'vertex_position_indices': [],
            'vertex_normals': [],
            'vertex_normal_indices': [],
            'vertex_uvs': [],
            'vertex_uv_indices': [],
            'texture_name': None,
            'armature_name': None,
            'vertex_group_indices': [],
            'vertex_group_weights': [],
            'num_groups_for_each_vertex': [],
            'bounding_box': {
                # [x, y, z]
                'min_corner': [],
                'max_corner': []
            }
        }

        if mesh.parent != None and mesh.parent.type == 'ARMATURE':
            mesh_json['armature_name'] = mesh.parent.name

        if mesh.data.uv_textures:
            texture_name = mesh.data.uv_textures.active.data[0].image.name
            mesh_json['texture_name'] = os.path.splitext(texture_name)[0]

        # TODO: Handle triangular polygons, not just quads
        # cube.data.polygons[1].vertices[0]. Check if length
        # of face is 4... Use a triangular face in Blender to unit test.
        index = 0
        for face in mesh.data.polygons:
            num_vertices_in_face = len(face.vertices)
            mesh_json['num_vertices_in_each_face'].append(num_vertices_in_face)

            for i in range(num_vertices_in_face):
                mesh_json['vertex_position_indices'].append(face.vertices[i])
                # TODO: Maintain a dictionary with (x, y, z) => normal index
                # for normals that we've already run into.
                # Re-use an existing normal index wherever possible. Especially important
                # for smoothed models that mostly re-use the same normals. Test this by
                # making a cube with to faces that have the same normal
                mesh_json['vertex_normal_indices'].append(index)
                if mesh.data.uv_textures:
                    mesh_json['vertex_uv_indices'].append(face.loop_indices[i])

            # TODO: Don't append normals if we've already encountered them
            mesh_json['vertex_normals'].append(face.normal.x)
            mesh_json['vertex_normals'].append(face.normal.y)
            mesh_json['vertex_normals'].append(face.normal.z)

            index += 1

        # TODO: Breadcrumb - iterate over the vertices and add them to mesh_json
        # TODO: Option for # of decimal places to round positions / normals / etc to.
        # Potentially just one option or a separate option for each
        for vert in mesh.data.vertices:
            mesh_json['vertex_positions'].append(vert.co.x)
            mesh_json['vertex_positions'].append(vert.co.y)
            mesh_json['vertex_positions'].append(vert.co.z)

            # TODO: Only include num groups if there is a parent armature. Otherwise the
            # number of groups (bones) per vertex probably doesn't matter...?
            num_groups = len(list(vert.groups))
            for group in vert.groups:
                mesh_json['vertex_group_indices'].append(group.group)
                mesh_json['vertex_group_weights'].append(group.weight)
                # groupName = mesh.vertex_groups[group.group].name

            if mesh_json['armature_name'] is not None:
                mesh_json['num_groups_for_each_vertex'].append(num_groups)

        if mesh.data.uv_textures:
            for loop in mesh.data.uv_layers.active.data:
                mesh_json['vertex_uvs'].append(loop.uv.x)
                mesh_json['vertex_uvs'].append(loop.uv.y)

        if mesh_json['armature_name'] == None:
            mesh_json['vertex_group_indices'] = None
            mesh_json['vertex_group_weights'] = None
            mesh_json['num_groups_for_each_vertex'] = None

        if mesh_json['texture_name'] == None:
            mesh_json['vertex_uvs'] = None
            mesh_json['vertex_uv_indices'] = None

        # TODO: Add unit test for no mesh currently selected
        # if mesh == None or mesh.type != 'MESH':
        #     print("__NO_MESH_SELECTED__", file=sys.stderr)
        #     return {'FINISHED'}

        # We construct our bounding box by iterating over all of the corners of the
        # mesh and finding the smallest and largest x, y and z values. Remember that we
        # are in a z up coordinate system in Blender.

        index = 0
        min_corner = [float('inf'), float('inf'), float('inf')];
        max_corner = [-float('inf'), -float('inf'), -float('inf')];

        # By switching to EDIT mode we'll ensure that our mesh is in its bind position.
        # Otherwise we might get the bounding box of the mesh while it was in the middle of some keyframe
        # which could be different from its bounding box in bind position.
        bpy.ops.object.mode_set(mode = 'EDIT')

        for corner in mesh.bound_box:
            # Get the Blender world space (within Blender) coordinates for the corner of this mesh.
            # This gives us the actual (x, y, z) coordinates of the corner in Blender's coordinate space,
            # instead of relative to the model's origin.
            # Modified from - https://blender.stackexchange.com/a/8470
            corner = Vector(corner)
            corner = mesh.matrix_world * corner

            # Min Corner
            min_corner[0] = min(min_corner[0], corner.x)
            min_corner[1] = min(min_corner[1], corner.y)
            min_corner[2] = min(min_corner[2], corner.z)
            # Max corner
            max_corner[0] = max(max_corner[0], corner.x)
            max_corner[1] = max(max_corner[1], corner.y)
            max_corner[2] = max(max_corner[2], corner.z)

        bpy.ops.object.mode_set(mode = 'OBJECT')
        mesh_json['bounding_box']['min_corner'] = min_corner
        mesh_json['bounding_box']['max_corner'] = max_corner

        # START_MESH_JSON $BLENDER_FILEPATH $MESH_NAME
        # ... mesh json ...
        # END_MESH_JSON $BLENDER_FILEPATH $MESH_NAME
        #
        # NOTE: Intentionally done in one print statement to get around
        # a bug where other Blender output (in this case from bpy.ops.anim.keyframe_delete(override, type='LocRotScale')
        # calls in blender-iks-to-fks) was getting mixed in with our JSON output
        output = "START_MESH_JSON " + bpy.data.filepath + " " + mesh.name
        output += "\n"
        output += json.dumps(mesh_json)
        output += "\n"
        output += "END_MESH_JSON " + bpy.data.filepath + " " + mesh.name
        print(output)

        return {'FINISHED'}

def register():
    bpy.utils.register_class(MeshToJSON)

def unregister():
    bpy.utils.unregister_class(MeshToJSON)

if __name__ == "__main__":
    register()

