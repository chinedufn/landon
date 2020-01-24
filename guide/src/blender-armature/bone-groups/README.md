# Bone Groups

`landon` exports all [bone groups] found on an armature.

One use case for this data is render different animations for different parts of an armature.

For example - if your game character is walking while punching - you might want to play a walking animation
on it's lower body bones and a punch animation on its upper body bones.

In Blender you'd create a bone group for the upper body and another bone group for the lower body and then
use this data in your renderer when determining which bones to render.

![Blender bone groups](./bone-groups.png)

### Rust API

https://docs.rs/blender-armature/0.2.0/blender_armature/struct.BlenderArmature.html#method.bone_groups

[bone groups]: https://docs.blender.org/manual/en/latest/animation/armatures/properties/bone_groups.html
