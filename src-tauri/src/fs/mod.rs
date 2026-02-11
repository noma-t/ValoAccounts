use std::ffi::OsStr;
use std::fs;
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use winapi::um::fileapi::{CreateFileW, GetFileAttributesW, INVALID_FILE_ATTRIBUTES, OPEN_EXISTING};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::ioapiset::DeviceIoControl;
use winapi::um::winbase::FILE_FLAG_BACKUP_SEMANTICS;
use winapi::um::winioctl::FSCTL_GET_REPARSE_POINT;
use winapi::um::winnt::{
    FILE_ATTRIBUTE_REPARSE_POINT, FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, HANDLE,
    MAXIMUM_REPARSE_DATA_BUFFER_SIZE,
};

const IO_REPARSE_TAG_MOUNT_POINT: u32 = 0xA0000003;

#[repr(C)]
struct ReparseDataBuffer {
    reparse_tag: u32,
    reparse_data_length: u16,
    reserved: u16,
    substitute_name_offset: u16,
    substitute_name_length: u16,
    print_name_offset: u16,
    print_name_length: u16,
    path_buffer: [u16; 1],
}

/// Check if a path is a junction point (symlink)
pub fn is_symlink(path: &Path) -> Result<bool, String> {
    let wide_path: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let attributes = GetFileAttributesW(wide_path.as_ptr());
        if attributes == INVALID_FILE_ATTRIBUTES {
            return Ok(false);
        }
        Ok(attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0)
    }
}

/// Create a junction point from `link` to `target`
pub fn create_junction(link: &Path, target: &Path) -> Result<(), String> {
    log::debug!("Creating junction: {} -> {}", link.display(), target.display());

    // Verify target exists
    if !target.exists() {
        log::error!("Target directory does not exist: {}", target.display());
        return Err(format!(
            "Target directory does not exist: {}",
            target.display()
        ));
    }

    if !target.is_dir() {
        log::error!("Target is not a directory: {}", target.display());
        return Err(format!("Target is not a directory: {}", target.display()));
    }

    // Verify link parent directory exists
    if let Some(parent) = link.parent() {
        if !parent.exists() {
            log::error!("Parent directory does not exist: {}", parent.display());
            return Err(format!(
                "Parent directory does not exist: {}",
                parent.display()
            ));
        }
    }

    // Check if link already exists
    if link.exists() {
        log::error!("Link path already exists: {}", link.display());
        return Err(format!("Link path already exists: {}", link.display()));
    }

    // Use junction.exe as a reliable method for creating junction points
    log::debug!("Executing mklink /J command");
    let output = std::process::Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            &link.to_string_lossy(),
            &target.to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("Failed to execute mklink command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::error!("Failed to create junction: {}", stderr);
        return Err(format!(
            "Failed to create junction from {} to {}: {}",
            link.display(),
            target.display(),
            stderr
        ));
    }

    log::info!("Junction created successfully: {} -> {}", link.display(), target.display());
    Ok(())
}

/// Remove a junction point
pub fn remove_junction(link: &Path) -> Result<(), String> {
    log::debug!("Removing junction: {}", link.display());

    if !link.exists() {
        log::debug!("Junction does not exist, skipping removal: {}", link.display());
        return Ok(());
    }

    if !is_symlink(link)? {
        log::error!("Path is not a junction point: {}", link.display());
        return Err(format!(
            "Path is not a junction point: {}",
            link.display()
        ));
    }

    fs::remove_dir(link).map_err(|e| {
        log::error!("Failed to remove junction point: {}", e);
        format!(
            "Failed to remove junction point {}: {}",
            link.display(),
            e
        )
    })?;

    log::info!("Junction removed successfully: {}", link.display());
    Ok(())
}

/// Get the target path of a junction point
pub fn get_junction_target(link: &Path) -> Result<PathBuf, String> {
    if !is_symlink(link)? {
        return Err(format!("Path is not a junction point: {}", link.display()));
    }

    let wide_path: Vec<u16> = OsStr::new(link)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let handle: HANDLE = CreateFileW(
            wide_path.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            std::ptr::null_mut(),
        );

        if handle == INVALID_HANDLE_VALUE {
            return Err(format!(
                "Failed to open junction point: {}",
                io::Error::last_os_error()
            ));
        }

        let mut buffer: Vec<u8> = vec![0; MAXIMUM_REPARSE_DATA_BUFFER_SIZE as usize];
        let mut bytes_returned: u32 = 0;

        let result = DeviceIoControl(
            handle,
            FSCTL_GET_REPARSE_POINT,
            std::ptr::null_mut(),
            0,
            buffer.as_mut_ptr() as *mut _,
            buffer.len() as u32,
            &mut bytes_returned,
            std::ptr::null_mut(),
        );

        CloseHandle(handle);

        if result == 0 {
            return Err(format!(
                "Failed to get reparse point data: {}",
                io::Error::last_os_error()
            ));
        }

        let reparse_data = &*(buffer.as_ptr() as *const ReparseDataBuffer);

        if reparse_data.reparse_tag != IO_REPARSE_TAG_MOUNT_POINT {
            return Err("Path is not a mount point".to_string());
        }

        let substitute_name_offset = reparse_data.substitute_name_offset as usize / 2;
        let substitute_name_length = reparse_data.substitute_name_length as usize / 2;

        let path_buffer_start = &reparse_data.path_buffer as *const u16;
        let substitute_name_slice = std::slice::from_raw_parts(
            path_buffer_start.add(substitute_name_offset),
            substitute_name_length,
        );

        let target_path = String::from_utf16_lossy(substitute_name_slice);

        // Remove \\?\  prefix if present
        let cleaned_path = target_path
            .strip_prefix(r"\\?\")
            .unwrap_or(&target_path)
            .to_string();

        Ok(PathBuf::from(cleaned_path))
    }
}

/// Create directory and place a marker file with the same name as the directory
/// This is useful for debugging to verify which directory is being used
pub fn create_dir_with_marker(dir_path: &Path) -> Result<(), String> {
    log::debug!("Creating directory with marker: {}", dir_path.display());

    fs::create_dir_all(dir_path)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // Create marker file with directory name (no extension)
    if let Some(dir_name) = dir_path.file_name() {
        let marker_file = dir_path.join(dir_name);
        log::debug!("Creating marker file: {}", marker_file.display());
        fs::write(&marker_file, "")
            .map_err(|e| format!("Failed to create marker file: {}", e))?;
    }

    Ok(())
}

/// Move all contents from source directory to destination directory
/// Uses copy-verify-delete pattern to prevent data loss
pub fn move_directory_contents(src: &Path, dest: &Path) -> Result<(), String> {
    log::info!("Moving directory contents: {} -> {}", src.display(), dest.display());

    if !src.exists() {
        log::error!("Source directory does not exist: {}", src.display());
        return Err(format!("Source directory does not exist: {}", src.display()));
    }

    if !src.is_dir() {
        log::error!("Source is not a directory: {}", src.display());
        return Err(format!("Source is not a directory: {}", src.display()));
    }

    // Create destination if it doesn't exist
    log::debug!("Creating destination directory: {}", dest.display());
    fs::create_dir_all(dest).map_err(|e| {
        format!(
            "Failed to create destination directory {}: {}",
            dest.display(),
            e
        )
    })?;

    // Read all entries
    let entries = fs::read_dir(src).map_err(|e| {
        format!(
            "Failed to read source directory {}: {}",
            src.display(),
            e
        )
    })?;

    let mut copied_entries = Vec::new();

    // Copy all entries
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dest.join(&file_name);

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path).map_err(|e| {
                format!(
                    "Failed to copy file from {} to {}: {}",
                    src_path.display(),
                    dest_path.display(),
                    e
                )
            })?;
        }

        copied_entries.push((src_path, dest_path));
    }

    // Verify all copies succeeded
    log::debug!("Verifying copied files");
    for (src_path, dest_path) in &copied_entries {
        if !dest_path.exists() {
            log::error!("Verification failed: destination file does not exist: {}", dest_path.display());
            return Err(format!(
                "Verification failed: destination file does not exist: {}",
                dest_path.display()
            ));
        }

        if src_path.is_file() {
            let src_metadata = fs::metadata(src_path).map_err(|e| {
                format!("Failed to read source metadata: {}", e)
            })?;
            let dest_metadata = fs::metadata(dest_path).map_err(|e| {
                format!("Failed to read destination metadata: {}", e)
            })?;

            if src_metadata.len() != dest_metadata.len() {
                log::error!("Verification failed: file size mismatch for {}", dest_path.display());
                return Err(format!(
                    "Verification failed: file size mismatch for {}",
                    dest_path.display()
                ));
            }
        }
    }

    log::debug!("Verification successful, deleting source entries");
    // Delete source entries only after verification
    for (src_path, _) in copied_entries {
        if src_path.is_dir() {
            fs::remove_dir_all(&src_path).map_err(|e| {
                format!(
                    "Failed to remove source directory {}: {}",
                    src_path.display(),
                    e
                )
            })?;
        } else {
            fs::remove_file(&src_path).map_err(|e| {
                format!(
                    "Failed to remove source file {}: {}",
                    src_path.display(),
                    e
                )
            })?;
        }
    }

    log::info!("Directory contents moved successfully");
    Ok(())
}

/// Helper function to recursively copy a directory
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| {
        format!(
            "Failed to create directory {}: {}",
            dest.display(),
            e
        )
    })?;

    let entries = fs::read_dir(src).map_err(|e| {
        format!("Failed to read directory {}: {}", src.display(), e)
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path).map_err(|e| {
                format!(
                    "Failed to copy file from {} to {}: {}",
                    src_path.display(),
                    dest_path.display(),
                    e
                )
            })?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_create_and_remove_junction() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("target");
        let link = temp_dir.path().join("link");

        fs::create_dir(&target).unwrap();

        // Create junction
        create_junction(&link, &target).unwrap();
        assert!(link.exists());
        assert!(is_symlink(&link).unwrap());

        // Remove junction
        remove_junction(&link).unwrap();
        assert!(!link.exists());
    }

    #[test]
    #[ignore]
    fn test_get_junction_target() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("target");
        let link = temp_dir.path().join("link");

        fs::create_dir(&target).unwrap();
        create_junction(&link, &target).unwrap();

        let retrieved_target = get_junction_target(&link).unwrap();
        assert_eq!(
            retrieved_target.canonicalize().unwrap(),
            target.canonicalize().unwrap()
        );

        remove_junction(&link).unwrap();
    }

    #[test]
    fn test_move_directory_contents() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        let dest = temp_dir.path().join("dest");

        fs::create_dir(&src).unwrap();
        fs::write(src.join("file1.txt"), "content1").unwrap();
        fs::write(src.join("file2.txt"), "content2").unwrap();

        let subdir = src.join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("file3.txt"), "content3").unwrap();

        move_directory_contents(&src, &dest).unwrap();

        assert!(dest.join("file1.txt").exists());
        assert!(dest.join("file2.txt").exists());
        assert!(dest.join("subdir").join("file3.txt").exists());

        assert!(!src.join("file1.txt").exists());
        assert!(!src.join("file2.txt").exists());
        assert!(!src.join("subdir").exists());
    }

    #[test]
    fn test_create_junction_with_nonexistent_target() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("nonexistent");
        let link = temp_dir.path().join("link");

        let result = create_junction(&link, &target);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Target directory does not exist"));
    }
}
