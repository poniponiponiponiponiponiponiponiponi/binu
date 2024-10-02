use std::io;
use std::path::Path;
use std::io::{Write, Read};
use std::fs::File;
use crate::util;

#[derive(Default, Debug)]
pub struct InsertConfig {
    pub quiet: bool,
}

/// Function for executing the command line insert command. You
/// probably want to use `insert()` instead.
pub fn insert_command(
    to_insert: &[u8],
    offset: usize,
    input_filename: &Path,
    output_filename: &Path,
    insert_config: &InsertConfig
) -> Result<(), io::Error> {
    insert(to_insert, offset, input_filename, output_filename)?;
    if !insert_config.quiet {
        println!("Inserting was successful");
    }
    
    Ok(())
}

/// Insert bytes from `to_insert` in offset specified in `offset`
/// counting from 0. Results are saved in `output_filename`.
pub fn insert(
    to_insert: &[u8],
    offset: usize,
    input_filename: &Path,
    output_filename: &Path,
) -> Result<(), io::Error> {
    let mut input_file = util::open_file(input_filename)?;
    let mut output_file = File::create(output_filename)?;

    // This will crash if there's not enough RAM but it's good enough for now.
    // Why Rust doesn't have sendfile(2)?!
    // A simple solution would be to read the file in chunks.
    let mut buf = vec![0u8; offset];
    input_file.file.read_exact(&mut buf)?;
    output_file.write_all(&buf)?;
    output_file.write_all(to_insert)?;
    let mut buf = Vec::new();
    input_file.file.read_to_end(&mut buf)?;
    output_file.write_all(&buf)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use crate::grep;

    #[test]
    fn simple_insert_test() {
        insert(
            b"meow",
            0,
            Path::new("test_files/file_three"),
            Path::new("test_files/file_three_insert")
        ).expect("Probably a file related error");

        let file = vec!["test_files/file_three_insert"];
        let res = grep::grep(b"meow", &file).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].1, vec![0]);

        let res = grep::grep(b"\x00", &file).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].1, vec![4, 5]);
    }
}
