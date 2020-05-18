# A script to temporarily install and run the addon. Useful for running
# blender-iks-to-fks via blender CLI where you might be in a
# continuous integration environment that doesn't have the addon
# installed
import bpy
import os

# Get the absolute path to the addon
dir = os.path.dirname(__file__)
addonFilePath = dir + '/convert-ik-to-fk.py'

# Install and enable the addon temporarily (since we aren't saving our user preferences)
# We just want to have access to the addon during this blender session
bpy.ops.preferences.addon_install(filepath=addonFilePath)
bpy.ops.preferences.addon_enable(module='convert-ik-to-fk')
# Run our addon
bpy.ops.rigging.iktofk()
