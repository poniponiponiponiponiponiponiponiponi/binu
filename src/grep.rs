use std::io;
use std::path::{PathBuf, Path};

use crate::util;

#[derive(Default, Debug)]
pub struct GrepConfig {
    pub quiet: bool,
    pub recursive: bool,
}

/// Function for executing the command line grep command. You probably
/// want to use `grep()` instead.
pub fn grep_command<T: AsRef<Path>>(
    pattern: &[u8],
    filenames: &[T],
    grep_config: &GrepConfig,
) -> Result<(), io::Error> {
    // Handle directories as paths
    let files;
    let paths: Vec::<&Path>;
    if grep_config.recursive {
        files = util::open_all_directories(filenames)?;
        paths = files.iter().map(|path| path.as_path()).collect::<Vec<&Path>>();
    } else {
        paths = filenames.iter().map(|path| path.as_ref()).collect();
    }

    // Get results
    let results = grep(pattern, &paths)?;
    let is_empty: bool = results.iter().all(|e| e.1.is_empty());
    if is_empty {
        if !grep_config.quiet {
            println!("Nothing found");
        }
        return Ok(());
    }

    // Pretty print
    for (n, (filename, offsets)) in results.iter().enumerate() {
        println!("{}:", filename.display());
        for (n, offset) in offsets.iter().enumerate() {
            print!("{}", offset);
            if n != offsets.len() - 1 {
                print!(", ");
            }
        }
        println!("{}", if n != results.len() - 1 {"\n"} else {""});
    }
    
    Ok(())
}

/// Find all occurrences of `pattern` in `filenames`. Return a Vec of
/// matched offsets.
pub fn grep<T: AsRef<Path>>(
    pattern: &[u8],
    filenames: &[T],
) -> Result<Vec<(PathBuf, Vec<u64>)>, io::Error> {
    let mut ret = Vec::new();
    for mut file in util::open_files(filenames) {
        ret.push((PathBuf::from(file.path), Vec::new()));
        let found_matches: Vec<_> = util::find_matches(&mut file, pattern).collect();
        for &offset in found_matches.iter() {
            ret.last_mut().unwrap().1.push(offset);
        }
    }
    
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_grep_test() {
        let files = vec!["test_files/file_one"];
        let res = grep(b"nya", &files).expect("Probably file not found");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].1, vec![
            3, 9, 12, 19, 22, 32, 43, 48, 55, 58, 64, 67, 74, 77, 84, 94, 104, 109
        ]);
    }

    #[test]
    fn simple_grep_test_two() {
        let files = vec!["test_files/file_three"];
        let res = grep(b"\x00", &files).expect("Probably file not found");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].1, vec![0, 1]);
    }

    #[test]
    fn simple_grep_test_multiple_files() {
        let files = vec!["test_files/file_one", "test_files/file_two", "test_files/file_three"];
        let res = grep(b"be", &files).expect("Probably file not found");
        assert_eq!(res.len(), 3);
        assert_eq!(res[0].1, vec![99]);
        assert_eq!(res[1].1, vec![12]);
        assert_eq!(res[2].1, vec![]);
    }
}
