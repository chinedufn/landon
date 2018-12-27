use crate::BlenderArmature;
use serde_json;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub type ArmaturesByFilename = HashMap<String, ArmaturesByArmatureName>;
pub type ArmaturesByArmatureName = HashMap<String, BlenderArmature>;

/// Given a buffer of standard output from Blender we parse all of the armature JSON that was
/// written to stdout by `blender-armature-to-json.py`.
///
/// Armaturees data in stdout will look like:
///
/// START_ARMATURE_JSON /path/to/file.blend my_armature_name
/// {...}
/// END_ARMATURE_JSON /path/to/file.blend my_armature_name
///
/// @see blender-armature-to-json.py - This is where we write to stdout
pub fn parse_armatures_from_blender_stdout(
    blender_stdout: &str,
) -> Result<ArmaturesByFilename, failure::Error> {
    let mut filenames_to_armature = HashMap::new();

    let mut index = 0;

    while let Some((filename_to_armature, next_start_index)) =
        find_first_armature_after_index(blender_stdout, index)
    {
        filenames_to_armature.extend(filename_to_armature);
        index = next_start_index;
    }

    Ok(filenames_to_armature)
}

/// Convert ArmatureeshByFilename into a HashMap<ArmatureName, BlenderArmature> that flattens all of the
/// armatures across all of the files into one HashMap.
///
/// This will error if there are two armatures with the same name across two or more files.
pub fn flatten_exported_armatures(
    armatures_by_filename: &ArmaturesByFilename,
) -> Result<HashMap<&str, &BlenderArmature>, FlattenArmatureError> {
    let mut flattened_armatures = HashMap::new();

    let mut duplicate_armatures: HashMap<String, Vec<String>> = HashMap::new();

    for (source_filename, armatures) in armatures_by_filename.iter() {
        for (armature_name, armature) in armatures.iter() {
            flattened_armatures.insert(armature_name.as_str(), armature);

            match duplicate_armatures.entry(armature_name.to_string()) {
                Entry::Occupied(mut duplicates) => {
                    duplicates.get_mut().push(source_filename.to_string());
                }
                Entry::Vacant(filenames) => {
                    filenames.insert(vec![source_filename.to_string()]);
                }
            };
        }
    }

    let duplicate_armatures: HashMap<String, Vec<String>> = duplicate_armatures
        .into_iter()
        .filter(|(_armature_name, filenames)| filenames.len() > 1)
        .collect();

    if duplicate_armatures.len() > 0 {
        return Err(FlattenArmatureError::DuplicateArmatureNamesAcrossFiles {
            duplicates: duplicate_armatures,
        });
    }

    Ok(flattened_armatures)
}

// FIXME: Move serde_json and parsing code behind a feature flag
fn find_first_armature_after_index(
    blender_stdout: &str,
    index: usize,
) -> Option<(ArmaturesByFilename, usize)> {
    let blender_stdout = &blender_stdout[index as usize..];

    if let Some(armature_start_index) = blender_stdout.find("START_ARMATURE_JSON") {
        let mut filenames_to_armature = HashMap::new();
        let mut armature_name_to_data = HashMap::new();

        let armature_end_index = blender_stdout.find("END_ARMATURE_JSON").unwrap();

        let armature_data = &blender_stdout[armature_start_index..armature_end_index];

        let mut lines = armature_data.lines();

        let first_line = lines.next().unwrap();

        let armature_filename: Vec<&str> = first_line.split(" ").collect();
        let armature_filename = armature_filename[1].to_string();

        let armature_name = first_line.split(" ").last().unwrap().to_string();

        let armature_data: String = lines.collect();
        let armature_data: BlenderArmature = serde_json::from_str(&armature_data).expect(&format!(
            "Could not deserialize Blender Armature data{}",
            &armature_data
        ));

        armature_name_to_data.insert(armature_name, armature_data);
        filenames_to_armature.insert(armature_filename, armature_name_to_data);

        return Some((filenames_to_armature, index + armature_end_index + 1));
    }

    return None;
}

/// An error when trying to flatten your exported data across multiple files into one HashMap of
/// armature name to armature data.
#[derive(Debug, Fail)]
pub enum FlattenArmatureError {
    #[fail(display = "Duplicate armatures found: {:#?}", duplicates)]
    DuplicateArmatureNamesAcrossFiles {
        // HashMap<ArmatureName, Vec<FilesThatItAppearsIn>>
        duplicates: HashMap<String, Vec<String>>,
    },
}
