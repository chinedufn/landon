use crate::Bone;

/// The transformation for a bone at a particular time
#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub struct BoneKeyframe {
    frame: u16,
    bone: Bone,
}

#[allow(missing_docs)]
impl BoneKeyframe {
    pub fn new(frame: u16, bone: Bone) -> Self {
        BoneKeyframe { frame, bone }
    }

    pub fn frame(&self) -> u16 {
        self.frame
    }

    pub fn bone(&self) -> Bone {
        self.bone
    }

    pub fn bone_mut(&mut self) -> &mut Bone {
        &mut self.bone
    }

    pub fn set_bone(&mut self, bone: Bone) {
        self.bone = bone;
    }
}
