# The goal of this addon is to export all of the actions for the active
# armature into a JSON file

import ast
import bpy
import collections
import json
import os
from mathutils import Vector

bl_info = {
    "name": "Export Mesh to JSON",
    "category": "Import-Export",
    "blender": (2, 80, 0)
}

# Write our JSON to stdout by default or to a file if specified.
# Stdout mesh JSON is wrapped in a start and end indicators
# to more easily distinguish it from other Blender output.
#
# START_MESH_JSON $BLENDER_FILEPATH $MESH_NAME
# ... mesh json ...
# END_MESH_JSON $BLENDER_FILEPATH $MESH_NAME
class MeshToJSON(bpy.types.Operator):
    """Given an active armature, export it's actions and keyframed bone
    pose information to a JSON file"""
    # Unique identifier for the addon
    bl_idname = 'import_export.mesh2json'
    # Display name in the interface
    bl_label = 'Export Mesh to JSON'
    bl_options = {'REGISTER'}
    bl_category = 'Import-Export'

    # The filepath to write out JSON to
    # filepath = bpy.props.StringProperty(name='filepath')

    def execute(self, context):
        bpy.ops.object.mode_set(mode='OBJECT')

        mesh = bpy.context.view_layer.objects.active

        mesh_json = {
            'name': mesh.name,
            'armature_name': None,
            # [x, y, z]
            'bounding_box': {
                'min_corner': [], 'max_corner': []
            },
            'materials': [],
            'custom_properties': {},
            'attribs': {
                'vertices_in_each_face': [],
                'material_index': [],
                'positions': {
                    'indices': [],
                    'attribute': {
                        'data': [],
                        'attribute_size': 3
                    }
                },
                'normals': {
                    'indices': [],
                    'attribute': {
                        'data': [],
                        'attribute_size': 3
                    }
                },
                'uvs': {
                    'indices': [],
                    'attribute': {
                        'data': [],
                        'attribute_size': 2
                    }
                },
                'bone_influences': {
                    'bones_per_vertex': {
                        'NonUniform': []
                    },
                    'bone_indices': [],
                    'bone_weights': []
                }
            }
        }

        # We maintain a list of all of the parent armature's bone names so that when exporting bone indices / weights
        # we use the same order that our armature use.
        # i.e. vertex group 12 that we export is the same as the 12th bone in the parent armature.
        # Without this the 12th vertex group on the mesh might actually be referring the 8th bone in the armature.
        # This would be a problem since our export format is currently based on the order of the bones in the armature.
        allBoneNames = []

        if mesh.parent is not None and mesh.parent.type == 'ARMATURE':
            parentArmature = mesh.parent
            mesh_json['armature_name'] = parentArmature.name
            for poseBone in parentArmature.pose.bones:
                allBoneNames.append(poseBone.name)

        # TODO: Handle triangular polygons, not just quads
        # cube.data.polygons[1].vertices[0]. Check if length
        # of face is 4... Use a triangular face in Blender to unit test.
        index = 0
        for face in mesh.data.polygons:
            num_vertices_in_face = len(face.vertices)
            mesh_json['attribs']['vertices_in_each_face'].append(num_vertices_in_face)
            mesh_json['attribs']['material_index'].append(face.material_index)

            for i in range(num_vertices_in_face):
                mesh_json['attribs']['positions']['indices'].append(face.vertices[i])
                # TODO: Maintain a dictionary with (x, y, z) => normal index
                # for normals that we've already run into.
                # Re-use an existing normal index wherever possible.
                # Especially important for smoothed models that mostly re-use
                # the same normals. Test this by making a cube with to faces
                # that have the same normal
                mesh_json['attribs']['normals']['indices'].append(index)
                if mesh.data.uv_layers:
                    mesh_json['attribs']['uvs']['indices'].append(face.loop_indices[i])

            # TODO: Don't append normals if we've already encountered them
            mesh_json['attribs']['normals']['attribute']['data'].append(face.normal.x)
            mesh_json['attribs']['normals']['attribute']['data'].append(face.normal.y)
            mesh_json['attribs']['normals']['attribute']['data'].append(face.normal.z)

            index += 1

        for vert in mesh.data.vertices:
            mesh_json['attribs']['positions']['attribute']['data'].append(vert.co.x)
            mesh_json['attribs']['positions']['attribute']['data'].append(vert.co.y)
            mesh_json['attribs']['positions']['attribute']['data'].append(vert.co.z)

            num_groups = len(list(vert.groups))
            for group in vert.groups:
                groupName = mesh.vertex_groups[group.group].name

                if groupName not in allBoneNames:
                    continue

                boneIndex = allBoneNames.index(groupName)

                mesh_json['attribs']['bone_influences']['bone_indices'].append(boneIndex)
                mesh_json['attribs']['bone_influences']['bone_weights'].append(group.weight)

            if mesh_json['armature_name'] is not None:
                mesh_json['attribs']['bone_influences']['bones_per_vertex']['NonUniform'].append(num_groups)

        if mesh.data.uv_layers:
            for loop in mesh.data.uv_layers.active.data:
                mesh_json['attribs']['uvs']['attribute']['data'].append(loop.uv.x)
                mesh_json['attribs']['uvs']['attribute']['data'].append(loop.uv.y)

        if not mesh_json['armature_name']:
            mesh_json['attribs']['bone_influences'] = None

        if not mesh_json['attribs']['uvs']['indices']:
            mesh_json['attribs']['uvs'] = None

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
            corner = mesh.matrix_world @ corner

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

        for material in mesh.data.materials:
            if material.node_tree == None:
                continue;

            # Iterate over the nodes until we find the Principled BSDF node. Then
            # read its properties
            for node in material.node_tree.nodes:
                baseColor = {}
                roughness = {}
                metallic = {}
                normalMap = None

                if node.type == 'BSDF_PRINCIPLED':
                    if len(node.inputs['Base Color'].links) > 0:
                        link = node.inputs['Base Color'].links[0]

                        # If there is a node feeding into the base_color, use
                        # that node's output color or image

                        if link.from_node.type == 'TEX_IMAGE':
                            baseColor['ImageTexture'] = link.from_node.image.name
                        else:
                            color = link.from_node.outputs['Color'].default_value
                            baseColor['Uniform'] = [
                                color[0], color[1], color[2]
                            ]
                    else:
                        # Otherwise use the output color set in the principled
                        # nodes color selector
                        color = node.inputs['Base Color'].default_value
                        baseColor['Uniform'] = [
                            color[0], color[1], color[2]
                        ]

                    if len(node.inputs['Roughness'].links) > 0:
                        link = node.inputs['Roughness'].links[0]

                        # If there is a node feeding into the roughness, use
                        # that node's output color or image

                        if link.from_node.type == 'TEX_IMAGE':
                            # If the channels weren't split, default to red
                            # channel
                            roughness['ImageTexture'] = [
                                link.from_node.image.name,
                                "R"
                            ]
                        elif link.from_node.type == 'SEPRGB':
                            print(mesh.name)
                            # example: ["some-texture.png", "R"]
                            roughness['ImageTexture'] = [
                                link.from_node.inputs['Image'].links[0].from_node.image.name,
                                link.from_socket.name # R, G or B
                            ]
                        else:
                            roughness['Uniform'] = link.from_node.outputs['Value'].default_value
                    else:
                        # Otherwise use the output color set in the principled
                        # nodes color selector
                        roughness['Uniform'] = node.inputs['Roughness'].default_value

                    if len(node.inputs['Metallic'].links) > 0:
                        link = node.inputs['Metallic'].links[0]

                        # If there is a node feeding into the metallic, use
                        # that node's output color or image

                        if link.from_node.type == 'TEX_IMAGE':
                            metallic['ImageTexture'] = [
                                # If the channels weren't split, default to
                                # green channel
                                link.from_node.image.name,
                                "G"
                            ]
                        elif link.from_node.type == 'SEPRGB':
                            # example: ["some-texture.png", "G"]
                            metallic['ImageTexture'] = [
                                link.from_node.inputs['Image'].links[0].from_node.image.name,
                                link.from_socket.name # R, G or B
                            ]
                        else:
                            metallic['Uniform'] = link.from_node.outputs['Value'].default_value

                    else:
                        # Otherwise use the output color set in the principled
                        # nodes color selector
                        metallic['Uniform'] = node.inputs['Metallic'].default_value

                    # Work backwards up to the normal map's image texture.
                    # Principled Node -> Normal Map -> Image Texture
                    if len(node.inputs['Normal'].links) > 0:
                        link = node.inputs['Normal'].links[0]

                        if link.from_node.type == 'NORMAL_MAP':
                            normalMapNode = link.from_node
                            normalMap = normalMapNode.inputs['Color'].links[0].from_node.image.name

                    mesh_json['materials'].append({
                        'name': material.name,
                        'base_color': baseColor,
                        'roughness': roughness,
                        'metallic': metallic,
                        'normal_map': normalMap
                    })

        for property in mesh.keys():
            # Not sure what this is but it gets automatically added into the properties. So we ignore it
            if property == '_RNA_UI':
                continue

            # Some properties such as 'cycles_visibility' are automatically inserted by Blender, but can't be
            # serialized.
            # Here we test if a property can be serialized, and if it can't we just skip it
            try:
                value = mesh.get(property)
                json.dumps(value)

                typed_value = {}
                try:
                    maybe_list = json.loads(value)
                    if isinstance(maybe_list, list):
                        typed_value = {"Vec": []}
                        for item in maybe_list:
                            if isinstance(item, float):
                                typed_value["Vec"].append({"Float": item})
                            elif isinstance(item, int):
                                typed_value["Vec"].append({"Int": item})
                            elif isinstance(item, str):
                                typed_value["Vec"].append({"String": item})
                        mesh_json['custom_properties'][property] = typed_value
                        continue
                except:
                    if isinstance(value, float):
                        typed_value = {"Float": value}
                    elif isinstance(value, int):
                        typed_value = {"Int": value}
                    elif isinstance(value, str):
                        typed_value = {"String": value}

                    mesh_json['custom_properties'][property] = typed_value
            except:
                pass

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

