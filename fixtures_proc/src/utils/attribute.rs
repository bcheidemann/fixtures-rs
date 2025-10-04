use syn::Path;

pub fn attribute_path_is<const N: usize>(path: &Path, segments: [&str; N]) -> bool {
    if path.leading_colon.is_some() {
        return false;
    }

    let mut actual_segments = path.segments.iter();

    for segment in segments.iter() {
        let Some(actual_segment) = actual_segments.next() else {
            return false;
        };

        if actual_segment.ident != *segment {
            return false;
        }
    }

    actual_segments.next().is_none()
}
