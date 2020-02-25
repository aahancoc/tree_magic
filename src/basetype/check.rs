use std::path::Path;
use crate::{read_bytes, MIME};

/// If there are any null bytes, return False. Otherwise return True.
fn is_text_plain_from_u8(b: &[u8]) -> bool
{
	b.iter().filter(|&x| *x == 0).count() == 0
}

// TODO: Hoist the main logic here somewhere else. This'll get redundant fast!
fn is_text_plain_from_filepath(filepath: &Path) -> bool
{
	let b = match read_bytes(filepath, 512) {
		Ok(x) => x,
		Err(_) => return false
	};
	is_text_plain_from_u8(b.as_slice())
}

#[allow(unused_variables)]
pub fn from_u8(b: &[u8], mimetype: MIME) -> bool
{
	if mimetype == "application/octet-stream" || 
	   mimetype == "all/allfiles"
	{
		// Both of these are the case if we have a bytestream at all
		return true;
	} if mimetype == "text/plain" {
		return is_text_plain_from_u8(b);
	} else {
		// ...how did we get bytes for this?
		return false;
	}
}

pub fn from_filepath(filepath: &Path, mimetype: MIME) -> bool
{
	use std::fs;

	// Being bad with error handling here,
	// but if you can't open it it's probably not a file.
	let meta = match fs::metadata(filepath) {
		Ok(x) => x,
		Err(_) => {return false;}
	};

	match mimetype.to_string().as_str() {
		"all/all" => return true,
		"all/allfiles" | "application/octet-stream" => return meta.is_file(),
		"inode/directory" => return meta.is_dir(),
		"text/plain" => return is_text_plain_from_filepath(filepath),
		_ => return false
	}
}