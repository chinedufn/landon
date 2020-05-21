# Troubleshooting

This chapter contains information about how to troubleshoot failed attempts to convert IK/FK rigs into FK rigs.

Over time we want to replace this information with code that makes it unnecessary.

## Unorganized

Here we dump troubleshooting information as we run into it that we can later clean up and organize into the right chapters.

### FK animation slightly off from IK animation

The FK animation isn't guaranteed to follow the same path as the original IK animation in between keyframes.

The IK rigs constraints might lead it to follow a certain path while the FK rig will simply have a roughly straight
interpolation between the keyframes.

To solve for this - run the IK to FK converter and then run both animations side by side in Blender.

If the FK rig is slightly out of place during a frame - insert a key at that frame on the original IK rig and
convert again.

In short - when the FK rig differs in interpolation from the IK rig it means you need more keyframes.
