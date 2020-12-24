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
                            locationsRotationsScales[boneName][frame] = {
                                "location": [0.0, 0.0, 0.0],
                                "rotation_quaternion": [1.0, 0.0, 0.0, 0.0],
                                "scale": [1.0, 1.0, 1.0],
                                "rotation_euler": [0.0, 0.0, 0.0]
                            }

                        locationsRotationsScales[boneName][frame][property][channel] = val

                for boneName, frames in locationsRotationsScales.items():
                    for frame, transforms in frames.items():
                        mat_scale = mathutils.Matrix.Scale(1.0, 4, transforms['scale'])
                        mat_loc = mathutils.Matrix.Translation(transforms['location'])

                        mat_rot = mathutils.Quaternion(transforms['rotation_quaternion']).to_matrix()

                        poseBone = allPoseBones[boneName]

                        if poseBone.rotation_mode != 'QUATERNION':
                            euler = transforms['rotation_euler']
                            mat_rot = mathutils.Euler((euler[0], euler[1], euler[2]), poseBone.rotation_mode).to_matrix()

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

                # Loop through the keyframes and build the frame data for the action
                #
                # TODO: Get bone pose matrices from fcurves (should be faster than querying for poses each frame)
                #  https://blenderartists.org/t/get-bone-position-data-matrix-relative-to-parent-bone/1116191/6
                #
                # TODO: Only insert a keyframe if the bone had a keyframe
                index = 0
                for frame in actionKeyframes:
                    # Get all of the bone pose matrices for this frame -> [bone1Matrix, bone2Matrix, ..]
                    armatureJSON['bone_space_actions'][actionInfo.name]['keyframes'].append({
                        'bones': [],
                        'frame': frame
                    })
                    for bone in getBonePosesAtKeyframe(frame, activeArmature, allBoneNames):
                        # https://docs.blender.org/api/current/bpy.types.PoseBone.html#bpy.types.PoseBone.matrix
                        boneWorldSpaceMatrix = activeArmature.matrix_world @ bone.matrix
                        armatureJSON['bone_space_actions'][actionInfo.name]['keyframes'][index]['bones'].append({'Matrix': matrixToArray(boneWorldSpaceMatrix)})

                    index += 1

                for pose_marker in activeArmature.animation_data.action.pose_markers:
                    armatureJSON['bone_space_actions'][actionInfo.name]['pose_markers'][pose_marker.frame] = pose_marker.name;

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

        # Get all of the bone pose matrices for the current keyframe
        # So if there are 10 bones, we'll get 10 matrices representing
        # these bones' orientations at this point in time
        def getBonePosesAtKeyframe(frame, armature, boneNames):
            bonePosesAtKeyframe = []
            bpy.context.scene.frame_set(frame)
            for boneName in boneNames:
                poseBone = armature.pose.bones[boneName]
                bonePosesAtKeyframe.append(poseBone)
            return bonePosesAtKeyframe

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
