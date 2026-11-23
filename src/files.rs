use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn is_image(path: &Path) -> bool {
    path.extension().and_then(|x| x.to_str()).is_some_and(|x| {
        matches!(
            x.to_ascii_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "bmp"
        )
    })
}

pub fn images_in(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_file() && is_image(&path) {
            paths.push(path);
        }
    }
    paths.sort_by_key(|p| p.file_name().map(|n| n.to_ascii_lowercase()));
    Ok(paths)
}

pub fn count_images(dir: &Path) -> io::Result<usize> {
    Ok(images_in(dir)?.len())
}

pub fn available_path(dir: &Path, name: &std::ffi::OsStr) -> io::Result<PathBuf> {
    let original = Path::new(name);
    let stem = original.file_stem().unwrap_or(name).to_string_lossy();
    let ext = original.extension().map(|x| x.to_string_lossy());
    let direct = dir.join(name);
    if !direct.exists() {
        return Ok(direct);
    }
    for i in 1..=100_000 {
        let candidate = match &ext {
            Some(ext) => dir.join(format!("{stem} ({i}).{ext}")),
            None => dir.join(format!("{stem} ({i})")),
        };
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "无法生成不重复的文件名",
    ))
}

pub fn move_file(source: &Path, target: &Path) -> io::Result<()> {
    if target.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "目标文件已存在",
        ));
    }
    match fs::rename(source, target) {
        Ok(()) => Ok(()),
        Err(e) if e.raw_os_error() == Some(18) || e.raw_os_error() == Some(17) => {
            let mut input = fs::File::open(source)?;
            let mut output = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(target)?;
            if let Err(err) = io::copy(&mut input, &mut output).and_then(|_| output.sync_all()) {
                drop(output);
                let _ = fs::remove_file(target);
                return Err(err);
            }
            drop(output);
            if let Err(err) = fs::remove_file(source) {
                let _ = fs::remove_file(target);
                return Err(err);
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn move_and_undo_with_collision() {
        let root = std::env::temp_dir().join(format!(
            "pixsort-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let src = root.join("src");
        let dst = root.join("dst");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&dst).unwrap();
        fs::write(src.join("a.jpg"), b"new").unwrap();
        fs::write(dst.join("a.jpg"), b"old").unwrap();
        let to = available_path(&dst, std::ffi::OsStr::new("a.jpg")).unwrap();
        assert_eq!(to.file_name().unwrap(), "a (1).jpg");
        move_file(&src.join("a.jpg"), &to).unwrap();
        assert_eq!(count_images(&dst).unwrap(), 2);
        move_file(&to, &src.join("a.jpg")).unwrap();
        assert_eq!(fs::read(src.join("a.jpg")).unwrap(), b"new");
        assert_eq!(fs::read(dst.join("a.jpg")).unwrap(), b"old");
        fs::remove_dir_all(root).unwrap();
    }
}
