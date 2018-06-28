# A script to temporarily install and run the addon. Useful for running
# blender-armature-to-json via blender CLI where you might be in a
# continuous integration environment that doesn't have the addon
# installed
#
# blender file.blend --python $(armature2json)
#  -> becomes ->
#    blender file.blend --python /path/to/run-addon
import bpy
import os

# Get the absolute path to the addon
dir = os.path.dirname(__file__)
addonFilePath = dir + '/src/blender-armature-to-json.py'

# Install and enable the addon temporarily (since we aren't saving our user preferences)
# We just want to have access to the addon during this blender session
bpy.ops.wm.addon_install(filepath=addonFilePath)
bpy.ops.wm.addon_enable(module='blender-armature-to-json')

print("ENABLED")
# Run our addon
bpy.ops.import_export.armature2json()
