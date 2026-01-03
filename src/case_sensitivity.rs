/*
 * modlem: A graphics importer/exporter for Lemmings
 * Copyright (C) 2022–2026 David Gow <david@davidgow.net>
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or (at your option) any later
 * version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
 * FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * this program. If not, see <https://www.gnu.org/licenses/>.
 */

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
