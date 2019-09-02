# A script to temporarily install and run the addon. Currently used
# by our tests but needs revisiting since I quickly threw it together.
#
# blender file.blend --python $(armature2json)
#  -> becomes ->
#    blender file.blend --python /path/to/run-addon
import bpy
import os

# Get the absolute path to the addon
dir = os.path.dirname(__file__)
addonFilePath = dir + '/../blender-armature-to-json.py'

# Install and enable the addon temporarily (since we aren't saving our user preferences)
# We just want to have access to the addon during this blender session
bpy.ops.preferences.addon_install(filepath=addonFilePath)
bpy.ops.preferences.addon_enable(module='blender-armature-to-json')

print("ENABLED")
# Run our addon
bpy.ops.import_export.armature2json()
