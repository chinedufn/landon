# Troubleshooting Rigify

## Shoulders and pelvis not converting properly

- Copy transforms constraint from pelvis.L to ORG-pelvis.L
- Copy transforms constraint from pelvis.R to ORG-pelvis.R
- Snap IK to FK (or vice versa) for all keyframes for both hands
  - Press `Action` in the `Rig Main properties` under `IK->FK` for left hand
  - Press `Action` in the `Rig Main properties` under `IK->FK` for right hand

- Action item: Open issue in the rigify repository asking if these constraints can be added by default

### Before Copying Transforms

Notice that the shoulders and pelvis are not green to start off with.

![Before copy transforms](./before-copy-transforms.png)

### After Copying Transforms

After copying transforms the shoulders and pelvis will become green.

![After copy transforms](./after-copy-transforms.png)
