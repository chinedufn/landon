import bpy

addonFilePath = '/tmp/blender-export/blender-armature-to-json.py'

# Install the addon, enable it and save the user's preferences so that it
# is available whenever Blender is opened in the future
bpy.ops.wm.addon_install(filepath=addonFilePath)
bpy.ops.wm.addon_enable(module='blender-armature-to-json')
bpy.ops.wm.save_userpref()
