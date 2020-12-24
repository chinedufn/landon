    # Print all of the actions for the active armature to stdout as JSON

bl_info = {
    "name": "Export Armature to JSON",
    "category": "Import-Export",
    "blender": (2, 80, 0)
}

import bpy
import math
import mathutils
import json

class ExportArmatureToJSON(bpy.types.Operator):
    """Given an active armature, export it's actions and keyframed bone pose information to a JSON file"""
    # Unique identifier for the addon
    bl_idname = 'import_export.armature2json'
    # Display name in the interface
    bl_label = 'Export Armature to JSON'
    bl_options = {'REGISTER'}
    bl_category = 'Import-Export'

    def execute(self, context):
        def main():
            # Get the armature that is currently active. We will be parsing it's actions
            # TODO: Error message if the active object is not the armature
            activeArmature = bpy.context.view_layer.objects.active
            # If the active object isn't an armature, we use the first armature that we find
            if activeArmature.type != 'ARMATURE':
                for obj in bpy.context.scene.objects:
                    if obj.type == 'ARMATURE':
                        activeArmature = obj
                        bpy.context.view_layer.objects.active = activeArmature
                        break

            armatureJSON = {
                'name': activeArmature.name,
                'bone_space_actions': {},
                'inverse_bind_poses': [],
                'joint_indices': {},
                'bone_child_to_parent': {},
                'bone_groups': {}
            }

            # Get all of the actions
            # TODO: If we later support handling multiple armatures we'll need to only use the
            # actions that apply to the current armature
            actionsList = list(bpy.data.actions)
            bpy.ops.object.mode_set(mode = 'POSE')

            # Select all of the armature's bones so that we can iterate over them later
            # We get all bone names so that we are certain to always iterate over bones in a consistent
            # order. We've had issues in the past where the order would be different depending on how you
            # accessed the bones, so this should help prevent future errors.
            allBoneNames = []
            allPoseBones = {}
            for poseBone in activeArmature.pose.bones:
                poseBone.bone.select = True
                allBoneNames.append(poseBone.name)
                allPoseBones[poseBone.name] = poseBone

            # Now we create the JSON for the joint name indices. The bind poses and keyframe poses are
            # arrays of index 0...numBones - 1. To look up a bone in this array you use its joint name index
            for boneIndex, boneName in enumerate(allBoneNames):
                armatureJSON['joint_indices'][boneName] = boneIndex

            # Parent indices
            for boneName in allBoneNames:
                poseBone = activeArmature.pose.bones[boneName]
                bone_idx = armatureJSON['joint_indices'][boneName]

                if poseBone.parent is not None:
                    parentIdx = armatureJSON['joint_indices'][poseBone.parent.name]
                    armatureJSON['bone_child_to_parent'][bone_idx] = parentIdx

            # Start building our JSON
            # The format is
            # {
            #   someAction: { timeInSeconds: [bone1, bone2, bone3 ...], keyframe2: [bone1, bone2, bone3 ...] },
            #   anotherAction: { someTime: [bone1, bone2, bone3 ...], keyframe2: [bone1, bone2, bone3 ...], anotherTime: { ... } },
            # }
            #
            # TODO: This is no longer the format. Re-write the docs after we port to Rust
            for actionInfo in actionsList:
                # Change to the action that we are currently parsing the data of
                activeArmature.animation_data.action = bpy.data.actions.get(actionInfo.name)
                action = activeArmature.animation_data.action

                locationsRotationsScales = {}

                # Get all of the keyframes for the current action. We'll iterate through them
                # to get all of the bone data
                actionKeyframes = getKeyframesInAction(activeArmature.animation_data.action)
                # If this action has no keyframes we skip it
                if actionKeyframes == []:
                    continue

                armatureJSON['bone_space_actions'][actionInfo.name] = {
                    'bone_keyframes': {
                        'frame_range_inclusive': [math.floor(action.frame_range[0]), math.floor(action.frame_range[1])],
                        'keyframes': {}
                    },
                    'keyframes': [],
                    'pose_markers': {}
                }

                # TODO: Cross reference our implementation with this:
                #  https://github.com/HENDRIX-ZT2/bfb-blender/blob/master/export_bf.py#L81
                for fcurve in action.fcurves:
                    # example: pose.bones["Lower.Body"].location
                    data_path = fcurve.data_path
                    channel = fcurve.array_index

                    prefix = 'pose.bones["'

                    if not data_path.startswith(prefix):
                        continue

                    path_pieces = data_path.replace(prefix, '').split('"].')

                    if len(path_pieces) < 2:
                        continue

                    # Lower.Body
                    boneName = path_pieces[0]
                    property = path_pieces[1]

                    if property not in ["location", "rotation_euler", "rotation_quaternion", "scale"]:
                        continue

                    if boneName not in allPoseBones:
                        continue

                    if boneName not in locationsRotationsScales:
                        locationsRotationsScales[boneName] = {}

                    for keyframe in fcurve.keyframe_points:
                        frame, val = keyframe.co

                        if frame not in locationsRotationsScales[boneName]:
                            locationsRotationsScales[boneName][frame] = {}

                        if property not in locationsRotationsScales[boneName][frame]:
                            locationsRotationsScales[boneName][frame][property] = {}

                        locationsRotationsScales[boneName][frame][property][channel] = val

                for boneName, frames in locationsRotationsScales.items():
                    previous_rot_euler_0 = None
                    previous_rot_euler_1 = None
                    previous_rot_euler_2 = None

                    previous_rot_quat_w = 1
                    previous_rot_quat_x = 0
                    previous_rot_quat_y = 0
                    previous_rot_quat_z = 0

                    previous_trans_x = 0
                    previous_trans_y = 0
                    previous_trans_z = 0

                    previous_scale_x = None
                    previous_scale_y = None
                    previous_scale_z = None

                    for frame in sorted(frames):
                        transforms = frames[frame]

                        mat_loc = mathutils.Matrix.Translation((0, 0, 0))
                        mat_rot = mathutils.Euler((0, 0, 0), 'XYZ').to_matrix()
                        mat_scale = mathutils.Matrix.Scale(1.0, 4, (1.0, 1.0, 1.0))

                        poseBone = allPoseBones[boneName]

                        # Handles Blender baking optimization that deletes redundant keyframes when neighbors share the
                        #  same value. We need to make sure to include the value in our transformation matrix

                        t = transforms.get('location') if transforms.get('location') is not None else {}
                        trans = [0, 0, 0]

                        trans[0] = t.get(0) if t.get(0) is not None else previous_trans_x
                        previous_trans_x = trans[0]

                        trans[1] = t.get(1) if t.get(1) is not None else previous_trans_y
                        previous_trans_y = trans[1]

                        trans[2] = t.get(2) if t.get(2) is not None else previous_trans_z
                        previous_trans_z = trans[2]

                        mat_loc = mathutils.Matrix.Translation(trans)

                        ### ---

                        if poseBone.rotation_mode == 'QUATERNION':
                            q = transforms.get('rotation_quaternion') if transforms.get('rotation_quaternion') is not None else {}
                            quat = [0, 0, 0, 0]

                            quat[0] = q.get(0) if q.get(0) is not None else previous_rot_quat_w
                            previous_rot_quat_w = quat[0]

                            quat[1] = q.get(1) if q.get(1) is not None else previous_rot_quat_x
                            previous_rot_quat_x = quat[1]

                            quat[2] = q.get(2) if q.get(2) is not None else previous_rot_quat_y
                            previous_rot_quat_y = quat[2]

                            quat[3] = q.get(3) if q.get(3) is not None else previous_rot_quat_z
                            previous_rot_quat_z = quat[3]

                            mat_rot = mathutils.Quaternion(quat).to_matrix()
                        else:
                            euler = transforms.get('rotation_euler') if transforms.get('rotation_euler') is not None else {}

                            euler[0] = euler.get(0) if euler.get(0) is not None else previous_rot_euler_0
                            previous_rot_euler_0 = euler[0]

                            euler[1] = euler.get(1) if euler.get(1) is not None else previous_rot_euler_1
                            previous_rot_euler_1 = euler[1]

                            euler[2] = euler.get(2) if euler.get(2) is not None else previous_rot_euler_2
                            previous_rot_euler_2 = euler[2]

                            mat_rot = mathutils.Euler((euler[0], euler[1], euler[2]), poseBone.rotation_mode).to_matrix()

                        # FIXME: Treat this like the above. Need to re-write and test all of this when we port
                        #  to Rust.
                        # if 'scale' in transforms:
                        #     mat_scale = mathutils.Matrix.Scale(1.0, 4, transforms['scale'])

                        mat_rot = mat_rot.to_4x4()

                        local_space_transform_matrix = mat_loc @ mat_rot @ mat_scale

                        bone_idx = armatureJSON['joint_indices'][boneName]
                        if bone_idx not in armatureJSON['bone_space_actions'][actionInfo.name]['bone_keyframes']['keyframes']:
                            armatureJSON['bone_space_actions'][actionInfo.name]['bone_keyframes']['keyframes'][bone_idx] = []

                        # bpy.context.scene.frame_set(frame)
                        armatureJSON['bone_space_actions'][actionInfo.name]['bone_keyframes']['keyframes'][bone_idx].append({
                            'frame': math.floor(frame),
                            'bone': {'Matrix': matrixToArray(local_space_transform_matrix)}
                        })


                for pose_marker in activeArmature.animation_data.action.pose_markers:
                    armatureJSON['bone_space_actions'][actionInfo.name]['pose_markers'][pose_marker.frame] = pose_marker.name

            # Calculate bone inverse bind poses
            for boneName in allBoneNames:
                # Calculate the bone's inverse bind matrix
                #
                # taken from:
                #   https://blenderartists.org/forum/showthread.php?323968-Exporting-armature-amp-actions-how-do-you-get-the-bind-pose-and-relative-transform
                #   https://blender.stackexchange.com/a/15229/40607
                poseBone = activeArmature.pose.bones[boneName]

                # We make sure to account for the world offset of the armature since matrix_local is in armature space
                # https://docs.blender.org/api/current/bpy.types.Bone.html#bpy.types.Bone.matrix_local
                boneBindMatrix = activeArmature.matrix_world @ poseBone.bone.matrix_local
                boneInverseBind = boneBindMatrix.copy().inverted()

                armatureJSON['inverse_bind_poses'].append({'Matrix': matrixToArray(boneInverseBind)})

            # Exporting bone groups
            #
            # 1. Deselect all bones in the armature
            # 2. Iterate through each bone group and select the bones in the group
            # 3. Add the selected bones to that bone group
            bone_groups_names = activeArmature.pose.bone_groups.keys()

            for poseBone in activeArmature.pose.bones:
                poseBone.bone.select = False

            for bone_group_index, bone_group_name in enumerate(bone_groups_names):
                armatureJSON['bone_groups'][bone_group_name] = []

                activeArmature.pose.bone_groups.active_index = bone_group_index

                bpy.ops.pose.group_select()

                for poseBone in bpy.context.selected_pose_bones:
                    bone_index = armatureJSON['joint_indices'][poseBone.bone.name]
                    armatureJSON['bone_groups'][bone_group_name].append(bone_index)

                bpy.ops.pose.group_deselect()

            # START_ARMATURE_JSON $BLENDER_FILEPATH $ARMATURE_NAME
            # ... mesh json ...
            # END_ARMATURE_JSON $BLENDER_FILEPATH $ARMATURE_NAME
            #
            # NOTE: Intentionally done in one print statement to get around
            # a bug where other Blender output (in this case from bpy.ops.anim.keyframe_delete(override, type='LocRotScale')
            # calls in blender-iks-to-fks) was getting mixed in with our JSON output
            output = "START_ARMATURE_JSON " + bpy.data.filepath + " " + activeArmature.name
            output += "\n"
            output += json.dumps(armatureJSON)
            output += "\n"
            output += "END_ARMATURE_JSON " + bpy.data.filepath + " " + activeArmature.name
            print(output)

            return {'FINISHED'}

        def getKeyframesInAction(action):
            # TODO: Right now we aren't sorting these keyframes.
            # We should sort them from lowest to highest (that's a more expected order).
            keyframes = []
            for fcurve in action.fcurves:
                for keyframe in fcurve.keyframe_points:
                    x, y = keyframe.co
                    # Don't know why yet, but we encounter each keyframes a
                    # bunch of times. so need to make sure we only add them once
                    if x not in keyframes:
                        # convert from float to int and insert into our keyframe list
                        keyframes.append((math.ceil(x)))
            return keyframes

        def matrixToArray (matrix):
            array = []
            for row in range(0, 4):
                for column in range(0, 4):
                    array.append(matrix[row][column])
            return array

        # Run our armature2json() add on
        return main()

def register():
    bpy.utils.register_class(ExportArmatureToJSON)

def unregister():
    bpy.utils.unregister_class(ExportArmatureToJSON)

if __name__ == "__main__":
    register()
