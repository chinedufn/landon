# The goal of this addon is to export all of the actions for the active
# armature into a JSON file

bl_info = {
    "name": "Export Mesh to JSON",
    "category": "Import-Export"
}

import bpy

class MeshToJSON(bpy.types.Operator):
    """Given an active armature, export it's actions and keyframed bone pose information to a JSON file"""
    # Unique identifier for the addon
    bl_idname = 'import_export.mesh2json'
    # Display name in the interface
    bl_label = 'Export Mesh to JSON'
    bl_options = {'REGISTER'}
    bl_category = 'Import-Export'

    # The filepath to write out JSON to
    filepath = bpy.props.StringProperty(name='filepath')
    # Write our JSON to stdout instead of writing it to a file
    # Stdout mesh JSON is wrapped in a start and end indicators
    # to more easily distinguish it from other Blender output
    #
    # ---START_EXPORT_MESH $MESH_NAME
    # ... mesh json ...
    # ---FINISH_EXPORT_MESH $MESH_NAME
    print_to_stdout = bpy.props.BoolProperty(name='stdout')

    def execute(self, context):
        print("\n\n\nHIHIHIH FROM BLENDER\n\n\n")
        return {'FINISHED'}

def register():
    bpy.utils.register_class(MeshToJSON)

def unregister():
    bpy.utils.unregister_class(MeshToJSON)

if __name__ == "__main__":
    register()

# >>> list(o.data.vertices[62].groups)[1].weight
# 0.32011678814888

# >>> list(o.data.vertices[62].groups)[1].group
# 6

# >>> list(o.data.vertices[62].groups)[0].group
# 5

# >>> o.vertex_groups
# bpy.data.objects['temp-player-full-body'].vertex_groups

# >>> o.vertex_groups[5]
# bpy.data.objects['temp-player-full-body']...VertexGroup

# >>> o.vertex_groups[5].name
# 'Lower.Arm.L'

# >>> o.vertex_groups[6].name
# 'Hand.L'
