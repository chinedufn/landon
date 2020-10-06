## 2.0.10

- Fix issue where conversions failed if the armature's name was different from its data block name [#15]

[#15]: https://github.com/chinedufn/landon/issues/15

## 1.4.2

- Fix issue where we weren't deleting all of the old keyframes

## 1.3.4

- If you have not selected a mesh we will automatically select the first mesh that we find that has an armature

## 1.3.0

- Allow addon to be installed via npm `npm install -g blender-iks-to-fks`
- Allow addon to be installed via command line: `ik2fk --install-blender`
- Allow addon to be run via command line
    ```
    blender my-file.blend --python `ik2fk`
    ```
- Add unit tests

## 1.2.0

- Previously your newly generated model would only be keyframed for the action that was active
when you ran `blender-iks-to-fks`. Now we loop through each action and bake all of your keyframes
onto your newly generated FK armature. This allows you to export all actions, not just to
one that you ran this script on

## 1.1.0

- Previously we would introduce new keyframes in between your existing ones
while baking keyframes. We now delete all newly generated keyframes before the
script ends, so that you're left with only your original keyframes.

## 1.0.0

- Initial release!
