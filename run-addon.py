# A script to temporarily install and run the addon. Useful for running
# blender-mesh-to-json via blender CLI where you might be in a
# continuous integration environment that doesn't have the addon
# installed
#
# blender file.blend --python $(mesh2json)
#  -> becomes ->
#    blender file.blend --python /path/to/run-addon
import bpy
import os

# Get the absolute path to the addon
dir = os.path.dirname(__file__)
addonFilePath = dir + '/blender-mesh-to-json.py'

# Install and enable the addon temporarily (since we aren't saving our user preferences)
# We just want to have access to the addon during this blender session
bpy.ops.preferences.addon_install(filepath=addonFilePath)
bpy.ops.preferences.addon_enable(module='blender-mesh-to-json')

# Run our addon
bpy.ops.import_export.mesh2json()
