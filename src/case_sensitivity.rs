/// Performs a case-insensitive search for a file in a given path, returning the case-corrected path.
pub fn find_file_in_dir(dir: &std::path::Path, name : &str) -> std::io::Result<std::path::PathBuf> {
	assert!(dir.is_dir());

	for filename in std::fs::read_dir(dir)? {
		let filename = filename?;
		let name_osstr = filename.file_name();
		if name_osstr.eq_ignore_ascii_case(name) {
			return Ok(filename.path());
		}
	}
	Err(std::io::Error::from(std::io::ErrorKind::NotFound))
}

/// Performs a case-insensitive search for a file in the current directory, returning the case-corrected path.
pub fn find_file_in_current_dir(name : &str) -> std::io::Result<std::path::PathBuf> {
	find_file_in_dir(&std::env::current_dir()?, name)
}
