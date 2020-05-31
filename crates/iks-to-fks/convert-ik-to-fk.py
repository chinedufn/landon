# The goal of this script is to convert an IK rig into an FK rig.
# We do this by creating a copy of your mesh and new FK rig for that mesh
#
# We do this by first duplicating our original rig and removing
# all of the IKs and constraints from our duplicate.
#
# We then bake the visual location and rotation into our FK rig
#
# Once this is done, the user can export their newly generated
# mesh and it's newly generated FK rig
#
# To run, install this file as an add on

bl_info = {
    "name": "Convert IKs to FKs",
    "category": "Rigging",
    "blender": (2, 80, 0)
}

import bpy

class ConvertIKToFK(bpy.types.Operator):
    """Given a selected mesh and armature with IKs and constraints, generate a new mesh and FK armature that has the same animations"""
    # Unique identifier for the add on
    bl_idname = 'rigging.iktofk'
    # Display name in the interface
    bl_label = 'Convert IKs to FKs'
    # Enable undo after executing the script
    bl_options = {'REGISTER', 'UNDO'}
    bl_category = 'Rigging'

    def execute(self, context):
        # We intentionally import bpy and math in here so that we can easily copy
        # and paste the contents of this execute function into the Blender python
        # console and have it work
        import bpy
        import sys
        import math

        ikArmature = None
        originalMesh = None

        # We first check if any of the already selected objects is a mesh that has a parent armature.
        # If so we use that mesh and armature
        for obj in bpy.context.selected_objects:
            if obj.type == 'MESH' and obj.parent and obj.parent.type == 'ARMATURE':
                originalMesh = obj
                ikArmature = obj.parent
                break

        # If no mesh is selected, we look for the first object that we can find that has an armature as a parent
        if originalMesh == None:
            for obj in bpy.data.objects:
                if obj.type == 'MESH' and obj.parent and obj.parent.type == 'ARMATURE':
                    originalMesh = obj
                    ikArmature = obj.parent
                    break

        # Deselect all objects and then select ONLY our armature and all of its children
        for obj in bpy.context.selected_objects:
            obj.select_set(state = False)

        if originalMesh != None and ikArmature != None:
            originalMesh.select_set(state=True)
            ikArmature.select_set(state=True)

        if ikArmature != None:
          # An active object is required in order to change into object mode
          bpy.context.view_layer.objects.active = ikArmature

          # If the armature is linked it will always be in object mode and we cannot mode_set.
          if ikArmature.library == None:
            bpy.ops.object.mode_set(mode = 'OBJECT')

          # Select our mesh and armature so that we can duplicate them later
          for mesh in ikArmature.children:
            if mesh.type == 'MESH':
              mesh.select_set(True)

        # Make sure that we have an armature selected
        if (len(list(bpy.context.selected_objects)) == 0):
            print('Error: File' + bpy.path.basename(bpy.context.blend_data.filepath) + ' does not have any armatures.', file=sys.stderr)

        originalActionsList = list(bpy.data.actions)

        # Duplicate the selected armature and mesh so that if anything were to go wrong there is a backup.
        #
        # Our duplicate (the current active armature) will become our new FK armature
        #
        # It's important to use our duplicate because our original armature might have special bones set up such as
        # boke hooks for a bezier curve. Trying to duplicate all of this onto a new armature would be difficult since
        # we'd need to also duplicate these bezier curves / bone hook data.
        bpy.ops.object.duplicate()
        fkArmature = bpy.context.view_layer.objects.active

        # Deselect all objects and then select ONLY our soon-to-be fkArmature
        for obj in bpy.context.selected_objects:
            obj.select_set(state = False)

        fkArmature.select_set(state=True)
        bpy.context.view_layer.objects.active = fkArmature

        # Enable all armature layers. Without this bones on disabled layers wouldn't get keyed.
        bpy.ops.object.mode_set(mode = 'EDIT')
        bpy.ops.armature.armature_layers(layers=(True, True, True, True, True, True, True, True, True, True, True, True, True, True, True, True, True, True, True, True, True, True, True, True, True, True, True, True, True, True, True, True))

        # Loop through all pose bones and make sure they are selected. Some of our commands require that the bones be selected
        bpy.ops.object.mode_set(mode = 'POSE')
        for poseBone in fkArmature.pose.bones:
            poseBone.bone.select = True

        # We iterate through the bones in the FK armature and remove all existing bone constraints
        for bone in fkArmature.pose.bones:
            for constraint in bone.constraints:
                bone.constraints.remove(constraint)

        # Now we remove all non deform bones from our FK armature
        bpy.ops.object.mode_set(mode = 'EDIT')
        for fkEditBone in bpy.data.armatures[fkArmature.name].edit_bones:
            if fkEditBone.use_deform == False:
                bpy.data.armatures[fkArmature.name].edit_bones.remove(fkEditBone)

         # Iterate through every action so that we can bake all keyframes across all actions
        for actionInfo in originalActionsList:
            # Next we make our FK bones copy the transforms of their IK rig counterparts
            # So bone1 in FK rig would copy transforms of bone1 in IK rig, and so on
            # We do this for every action since we clear our transforms while baking
            # visual keys below
            bpy.ops.object.mode_set(mode = 'POSE')
            for fkBone in bpy.context.selected_pose_bones:
                copyTransforms = fkBone.constraints.new('COPY_TRANSFORMS')
                copyTransforms.target = ikArmature
                # the name of the bone in our original armature is the same as the name of our
                # fkArmature bone the armature was duplicated. Therefore we us `fkBone.name`
                copyTransforms.subtarget = fkBone.name

            # Now that our FK rig is copying our IK rigs transforms, we insert visual keyframes
            # for every keyframe. This gives our FK rigs the IK rigs transforms, after
            # which we can then delete the IK rig
            bpy.ops.object.mode_set(mode = 'OBJECT')

            # Change to the action that we want to mimic
            ikArmature.animation_data.action = bpy.data.actions.get(actionInfo.name)
            fkArmature.animation_data.action = bpy.data.actions.get(actionInfo.name)

            # Get all of the keyframes that are set for the rigs
            keyframes = []
            for fcurve in bpy.context.active_object.animation_data.action.fcurves:
                for keyframe in fcurve.keyframe_points:
                    x, y = keyframe.co
                    # Don't know why yet, but we encounter each keyframes a
                    # bunch of times. so need to make sure we only add them once
                    if x not in keyframes:
                      # convert from float to int and insert into our keyframe list
                      keyframes.append((math.ceil(x)))
            # If this action has no keyframes we skip it
            if keyframes == []:
                 continue

            # Keyframes might not always be in order so we sort them
            keyframes.sort()

            # Now we bake all of our keyframes and remove our copy transform constraints
            bpy.ops.nla.bake(frame_start=keyframes[0], frame_end=keyframes[-1], only_selected=True, visual_keying=True, clear_constraints=True, use_current_action=True, bake_types={'POSE'})

            # Bake adds extra keyframes, so we delete any keyframes that did not previously exist
            bpy.ops.object.mode_set(mode = 'POSE')
            # Delete generated keyframes that did not exist before this script
            for frame in range(keyframes[0], keyframes[-1]):
                if frame not in keyframes:
                    bpy.context.scene.frame_set(frame)

                    # We set up the proper context to override the default for keyframe_delete.
                    # This fixes an issue where the `poll()` function on keyframe_delete was failing when run via blender CLI.
                    # In short.. we're just making sure that `keyframe_delete` uses the correct context
                    # When we run this addon from the command line.
                    screen = bpy.context.window.screen
                    for area in screen.areas:
                        if area.type == 'VIEW_3D':
                            for region in area.regions:
                                if region.type == 'WINDOW':
                                    override = {'window': bpy.context.window, 'screen': screen, 'area': area, 'region': region, 'scene': bpy.context.scene, 'active_object': bpy.context.active_object, 'active_pose_bone': bpy.context.active_pose_bone, 'selected_pose_bones': bpy.context.selected_pose_bones}
                                    bpy.ops.anim.keyframe_delete(override, type='LocRotScale')

        # Delete all of the actions that were created when we duplicate our mesh and armature
        #
        # Unless the IK armature was linked. In which case we need to keep our new actions
        # because our linked ones were not modified and do not have our new data.
        if ikArmature.library is None:
            for action in bpy.data.actions:
                if action not in originalActionsList:
                    bpy.data.actions.remove(action)
        else:
            for action in bpy.data.actions:
                if action.library != None:
                    bpy.data.actions.remove(action)

        # Go to Object mode so that they can export their new model
        bpy.ops.object.mode_set(mode = 'OBJECT')

        return {'FINISHED'}

def register():
    bpy.utils.register_class(ConvertIKToFK)

def unregister():
    bpy.utils.unregister_class(ConvertIKToFK)

# This allows you to run the script directly from blenders text editor
# to test the addon without having to install it as an add on.
# Hit `space` then search for `convert Iks to Fks`
#
# Alternatively, you can paste the contents of the execute script
# into your Blender Python console, just make sure to remove all `return`
# statements first
#
# This is only useful for testing while developing. Otherwise just use the
# official release from GitHub:
#   https://github.com/chinedufn/blender-iks-to-fks/releases/
if __name__ == "__main__":
    register()
