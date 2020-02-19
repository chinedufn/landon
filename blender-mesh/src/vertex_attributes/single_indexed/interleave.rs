use crate::vertex_attributes::VertexAttribute;
use crate::SingleIndexedVertexAttributes;

/// An error while interleaving vertex data
#[derive(Debug, thiserror::Error)]
pub enum InterleaveError {
    /// Likely a mistake when creating the slice of vertex attributes
    #[error("Only {num_provided} buffers were provided so there is nothing to interleave.")]
    RequiresAtLeastTwoBuffers { num_provided: usize },
    /// If, say, positions have an attribute size of 3 and uvs have a size of 2 then
    /// positions.len() / 3 should equal uvs.len() / 2
    #[error("The length of each attribute should correspond to the same number of vertices")]
    MismatchedLengths,
}

impl SingleIndexedVertexAttributes {
    /// Combine anu number of vertex attributes into a single buffer of vertex data.
    ///
    /// Say you have
    ///   positions: [0., 1., 2., 10., 11., 12.] with attribute size 3
    ///   uvs      : [0., 1., 1., 1.]
    ///
    /// This would get stitched together as
    ///              [0., 1., 2., 0., 1., 10., 11., 12., 1., 1.]
    ///
    /// More generally, say you have attributes P with size 3, U with size 2, N with size 3.
    ///
    /// They'll get interleaved as
    ///         [
    ///             P0, P0, P0, U0, U0, N0, N0,
    ///             P1, P1, P1, U1, U1, N1, N1, ...
    ///         ],
    pub fn interleave<T: Copy>(attribs: &[&VertexAttribute<T>]) -> Result<Vec<T>, InterleaveError> {
        if attribs.len() < 2 {
            return Err(InterleaveError::RequiresAtLeastTwoBuffers {
                num_provided: attribs.len(),
            });
        }

        let vertex_count = attribs[0].data.len() as f32 / attribs[0].attribute_size as f32;

        if !attribs
            .iter()
            .all(|attrib| attrib.len() as f32 / attrib.attribute_size as f32 == vertex_count)
        {
            return Err(InterleaveError::MismatchedLengths);
        }

        let vertex_count = vertex_count as usize;

        // TODO: We can by setting the vector to the correct capacity and length with uninitialized
        // memory and just iterate through each attribute and put the data in the right place with
        // the right stride

        let mut interleaved = vec![];

        for vertex in 0..vertex_count {
            for attrib in attribs {
                let attribuze_size = attrib.attribute_size as usize;
                let index = vertex * attribuze_size;
                for idx in index..index + attribuze_size {
                    interleaved.push(attrib.data[idx]);
                }
            }
        }

        Ok(interleaved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Interleave two vertex attributes
    #[test]
    fn combine_two_attributes() {
        let positions = VertexAttribute::new(vec![0., 1., 2., 3., 4., 5.], 3).unwrap();
        let uvs = VertexAttribute::new(vec![50., 51., 52., 53.], 2).unwrap();

        let combined = SingleIndexedVertexAttributes::interleave(&[&positions, &uvs]).unwrap();

        assert_eq!(combined, vec![0., 1., 2., 50., 51., 3., 4., 5., 52., 53.]);
    }

    /// Trying to interleave one buffer is likely a mistake
    #[test]
    fn only_one_buffer_provided() {
        let positions = VertexAttribute::new(vec![0., 1., 2., 3., 4., 5.], 3).unwrap();
        match SingleIndexedVertexAttributes::interleave(&[&positions]) {
            Err(InterleaveError::RequiresAtLeastTwoBuffers { num_provided: 1 }) => {}
            _ => unreachable!(),
        };
    }

    /// The lengths of all of the attributes should correspond to the same number of vertices
    #[test]
    fn error_if_incompatible_lengths() {
        let positions = VertexAttribute::new(vec![0.], 3).unwrap();
        let uvs = VertexAttribute::new(vec![50., 51., 52., 53.], 2).unwrap();

        match SingleIndexedVertexAttributes::interleave(&[&positions, &uvs]) {
            Err(InterleaveError::MismatchedLengths {}) => {}
            _ => unreachable!(),
        };
    }
}
