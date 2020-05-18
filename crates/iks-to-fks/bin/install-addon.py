# Install the addon and save the user's preferences
import bpy
import os

# Get the absolute path to the addon
dir = os.path.dirname(__file__)
addonFilePath = dir + '/../convert-ik-to-fk.py'

# Install the addon, enable it and save the user's preferences so that it
# is available whenever Blender is opened in the future
bpy.ops.preferences.addon_install(filepath=addonFilePath)
bpy.ops.preferences.addon_enable(module='convert-ik-to-fk')
bpy.ops.wm.save_userpref()
