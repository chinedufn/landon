# Install the addon and save the user's preferences
import bpy
import os

# Get the absolute path to the addon
dir = os.path.dirname(__file__)
addonFilePath = dir + '/blender-mesh-to-json.py'

# Install the addon, enable it and save the user's preferences so that it
# is available whenever Blender is opened in the future
bpy.ops.wm.addon_install(filepath=addonFilePath)
bpy.ops.wm.addon_enable(module='blender-mesh-to-json')
bpy.ops.wm.save_userpref()
