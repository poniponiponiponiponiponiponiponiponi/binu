use std::io;
use std::io::{Write, Seek, Read};
use std::fs::File;
use std::path::Path;
use crate::util;

#[derive(Default, Debug)]
pub struct ReplaceConfig {
    pub quiet: bool,
    pub nth: usize,
    pub replace_all: bool,
    pub fill_byte: u8,
    pub allow_length_change: bool,
}

/// Function for executing the command line replace command. You
/// probably want to use `replace()` instead.
pub fn replace_command(
    to_replace: &[u8],
    replace_with: &[u8],
    input_filename: &Path,
    output_filename: &Path,
    replace_config: &ReplaceConfig,
) -> Result<(), io::Error> {
    if !replace_config.allow_length_change && replace_with.len() > to_replace.len() {
        eprintln!("Replacing string is too long");
    }
    
    let n = replace(to_replace, replace_with, input_filename, output_filename, replace_config)?;
    if !replace_config.quiet {
        if n == 1 {
            println!("Replaced 1 match successfully");
        } else {
            println!("Replaced {} matches successfully", 0);
        }
    }
    
    Ok(())
}

/// Replace the `to_replace` pattern in the file `input_filename` with
/// bytes specified by `replace_with`. The result in saved in
/// `output_filename`. Return the number of replaced patterns
pub fn replace(
    to_replace: &[u8],
    replace_with: &[u8],
    input_filename: &Path,
    output_filename: &Path,
    replace_config: &ReplaceConfig,
) -> Result<usize, io::Error> {
    let mut input_file = util::open_file(input_filename)?;
    
    let mut matches_iter = util::find_matches(&mut input_file, to_replace);
    let found_matches: Vec<_>;

    // Make it so later replacing the matches is a generic case,
    // no matter if we're replacing one instance or all instances
    if !replace_config.replace_all {
        if let Some(offset) = matches_iter.nth(replace_config.nth) {
            found_matches = vec![offset];
        } else {
            return Ok(0);
        }
    } else {
        found_matches = matches_iter.collect()
    }

    // Initialize variables for the loop
    let to_fill = if replace_config.allow_length_change {
        0
    } else {
        to_replace.len() - replace_with.len()
    };
    let mut input_file = File::open(input_filename)?;
    let mut output_file = File::create(output_filename)?;
    let mut last_offset = 0;
    
    // Handle replacing the file with copying in this kind of pattern:
    // file[0:1st_off] + replace_with + file[1st_off+len(replace_with):2nd_off] + ...
    // Hope you see it, otherwise I don't know how to explain it better with words
    for &offset in found_matches.iter() {
        if last_offset > offset as usize {
            continue;
        }
        
        let mut buf = vec![0u8; offset as usize-last_offset];
        input_file.read_exact(&mut buf)?;
        output_file.write(&buf)?;
        
        input_file.seek_relative(to_replace.len() as i64)?;
        output_file.write(&replace_with)?;
        let fill_bytes = vec![replace_config.fill_byte; to_fill];
        output_file.write(&fill_bytes)?;

        last_offset += buf.len() + to_replace.len();
    }
    // Handle the last case which is from the last offset to the end of the file
    let mut buf = String::new();
    input_file.read_to_string(&mut buf)?;
    output_file.write(buf.as_bytes())?;
    
    Ok(found_matches.len())
}
