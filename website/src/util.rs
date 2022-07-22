use std::path::{Component, Path, PathBuf};

pub fn join_absolute_paths<P, Q>(head: P, tail: Q) -> Result<PathBuf, crate::Error>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let mut tail = tail.as_ref();

    // The Path extractor extracts the path including a leading `/`. This breaks
    // when concatenating it with the static dir, so make it relative. See the
    // docs for `Path.join`.
    if tail.is_absolute() {
        tail = tail.strip_prefix(Component::RootDir)?;
        debug_assert!(tail.is_relative());
    }

    Ok(head.as_ref().join(tail))
}
