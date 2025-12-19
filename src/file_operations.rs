use std::path::Path;

#[derive(Debug)]
pub struct FilePathConfig {
    pub unix_absolute_path: bool,
    pub windows_absolute_path: bool,
    pub relative_path_with_separators: bool,
    pub bare_filename: bool,
}

// prevents downstream problems with path.parent() when passing
// in a bare filename, such as measured.s2p
// this function help us adjust to ./measured.s2p so logic is easier later
pub fn get_file_path_config(path_str: &str) -> FilePathConfig {
    let path = Path::new(path_str);
    let mut unix_absolute_path = false;
    let mut windows_absolute_path = false;
    let mut relative_path_with_separators = false;
    let mut bare_filename = false;

    // Manual detection to be cross-platform safe
    // 1. Unix Absolute
    if path_str.starts_with('/') {
        unix_absolute_path = true;
    }
    // 2. Windows Absolute
    // Check for drive letter (e.g., C:\ or C:/)
    else if path_str.len() >= 3
        && path_str.chars().next().unwrap().is_ascii_alphabetic()
        && path_str.chars().nth(1).unwrap() == ':'
        && (path_str.chars().nth(2).unwrap() == '\\' || path_str.chars().nth(2).unwrap() == '/')
        // Check for UNC path (e.g. \\server or //server, but // could be multiple slashes on unix so be careful)
        // On Unix, // is just root, mostly. But usually we want to treat \\ as windows UNC.
        || path_str.starts_with(r"\\")
    {
        windows_absolute_path = true;
    }
    // 3. Relative with separators (nested)
    else if path.components().count() > 1 {
        // files/measured.s2p, etc.
        relative_path_with_separators = true;
    }
    // 4. Bare filename
    else {
        // measured.s2p, etc.
        bare_filename = true;
    }

    if unix_absolute_path {
        println!("'{}' is a Unix Absolute path.", path_str);
    } else if windows_absolute_path {
        println!("'{}' is a Windows Absolute path.", path_str);
    } else if relative_path_with_separators {
        println!(
            "'{}' is a Relative path with separators (nested).",
            path_str
        );
    } else {
        println!("'{}' is a Bare filename (no separators).", path_str);
    }

    FilePathConfig {
        unix_absolute_path,
        windows_absolute_path,
        relative_path_with_separators,
        bare_filename,
    }
}

fn path_to_url_manual(path_str: &str) -> String {
    // 1. Unify separators to '/'
    let cleaned_path = path_str.replace('\\', "/");

    // 2. URL encode spaces and other special characters
    let encoded_path = cleaned_path.replace(' ', "%20");

    // 3. Handle Windows vs Unix prefix
    if encoded_path.starts_with('/') {
        // Unix: just add file://
        format!("file://{}", encoded_path)
    } else {
        // Windows: C:/Path -> file:///C:/Path (needs 3 slashes)
        format!("file:///{}", encoded_path)
    }
}

pub fn get_file_url(file_path: &String) -> String {
    println!("file_path in get_file_url function: {}", file_path);
    let mut path_str: String = std::fs::canonicalize(file_path)
        .unwrap()
        .display()
        .to_string();

    // Remove the UNC prefix on Windows if present
    if cfg!(target_os = "windows") && path_str.starts_with(r"\\?\") {
        path_str = path_str[4..].to_string();
    }

    // add file_prefix
    path_to_url_manual(&path_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_path_config_absolute_path() {
        let config = get_file_path_config("/home/user/files/measured.s2p");
        assert!(config.unix_absolute_path);
        assert!(!config.windows_absolute_path);
        assert!(!config.relative_path_with_separators);
        assert!(!config.bare_filename);
    }

    #[test]
    fn test_get_file_path_config_relative_path_with_separators() {
        let config = get_file_path_config("files/measured.s2p");
        assert!(!config.unix_absolute_path);
        assert!(!config.windows_absolute_path);
        assert!(config.relative_path_with_separators);
        assert!(!config.bare_filename);
    }

    #[test]
    fn test_get_file_path_config_bare_filename() {
        let config = get_file_path_config("measured.s2p");
        assert!(!config.unix_absolute_path);
        assert!(!config.windows_absolute_path);
        assert!(!config.relative_path_with_separators);
        assert!(config.bare_filename);
    }

    #[test]
    fn test_get_file_path_config_nested_relative_path() {
        let config = get_file_path_config("a/b/c/measured.s2p");
        assert!(!config.unix_absolute_path);
        assert!(!config.windows_absolute_path);
        assert!(config.relative_path_with_separators);
        assert!(!config.bare_filename);
    }

    #[test]
    fn test_get_file_path_config_windows_absolute_path() {
        // On Unix systems, Windows paths with backslashes are treated as bare filenames
        // because backslash is not a path separator on Unix
        let config = get_file_path_config("C:\\Users\\test\\file.s2p");
        assert!(!config.unix_absolute_path);
        assert!(config.windows_absolute_path);
        assert!(!config.relative_path_with_separators);
        assert!(!config.bare_filename);
    }

    #[test]
    fn test_path_to_url_manual_unix_absolute() {
        let url = path_to_url_manual("/home/user/files/measured.s2p");
        assert_eq!(url, "file:///home/user/files/measured.s2p");
    }

    #[test]
    fn test_path_to_url_manual_windows_relative() {
        let url = path_to_url_manual("C:/Users/test/file.s2p");
        assert_eq!(url, "file:///C:/Users/test/file.s2p");
    }

    #[test]
    fn test_path_to_url_manual_backslash_conversion() {
        let url = path_to_url_manual("C:\\Users\\test\\file.s2p");
        assert_eq!(url, "file:///C:/Users/test/file.s2p");
    }

    #[test]
    fn test_path_to_url_manual_relative_unix_path() {
        let url = path_to_url_manual("files/measured.s2p");
        assert_eq!(url, "file:///files/measured.s2p");
    }

    #[test]
    fn test_path_to_url_manual_bare_filename() {
        let url = path_to_url_manual("measured.s2p");
        assert_eq!(url, "file:///measured.s2p");
    }

    #[test]
    fn test_get_file_path_config_windows_unc_path() {
        // Windows UNC path like \\server\mount\folder\file.s2p
        // On Unix systems, this is treated as a bare filename because
        // backslashes are not path separators on Unix
        let config = get_file_path_config("\\\\server\\mount\\folder\\file.s2p");
        assert!(!config.unix_absolute_path);
        assert!(config.windows_absolute_path);
        assert!(!config.relative_path_with_separators);
        assert!(!config.bare_filename);
    }

    #[test]
    fn test_path_to_url_manual_windows_unc_path() {
        // Windows UNC path: \\server\mount\folder\file.s2p
        // Should convert backslashes to forward slashes
        // and format as file:// URL
        let url = path_to_url_manual("\\\\server\\mount\\folder\\file.s2p");
        assert_eq!(url, "file:////server/mount/folder/file.s2p");
    }

    #[test]
    fn test_path_to_url_manual_windows_unc_path_forward_slashes() {
        // Windows UNC path with forward slashes
        let url = path_to_url_manual("//server/mount/folder/file.s2p");
        assert_eq!(url, "file:////server/mount/folder/file.s2p");
    }

    #[test]
    fn test_path_to_url_manual_unix_path_with_spaces() {
        // Unix path with spaces should be URL encoded
        let url = path_to_url_manual("/home/user/my files/measured.s2p");
        assert_eq!(url, "file:///home/user/my%20files/measured.s2p");
    }

    #[test]
    fn test_path_to_url_manual_windows_path_with_spaces() {
        // Windows path with spaces should be URL encoded
        let url = path_to_url_manual("C:\\Program Files\\test folder\\file.s2p");
        assert_eq!(url, "file:///C:/Program%20Files/test%20folder/file.s2p");
    }

    #[test]
    fn test_path_to_url_manual_bare_filename_with_spaces() {
        // Bare filename with spaces should be URL encoded
        let url = path_to_url_manual("my file.s2p");
        assert_eq!(url, "file:///my%20file.s2p");
    }

    #[test]
    fn test_get_file_path_config_filename_with_spaces() {
        // Filename with spaces should be recognized as bare filename
        let config = get_file_path_config("my measurements.s2p");
        assert!(!config.unix_absolute_path);
        assert!(!config.windows_absolute_path);
        assert!(!config.relative_path_with_separators);
        assert!(config.bare_filename);
    }

    #[test]
    fn test_get_file_path_config_relative_path_with_spaces() {
        // Relative path with spaces should be recognized with separators
        let config = get_file_path_config("my files/measurements.s2p");
        assert!(!config.unix_absolute_path);
        assert!(!config.windows_absolute_path);
        assert!(config.relative_path_with_separators);
        assert!(!config.bare_filename);
    }
}
