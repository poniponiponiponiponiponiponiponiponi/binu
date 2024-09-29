use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{PathBuf, Path};

#[derive(Default, Debug)]
pub struct GrepConfig {
    pub quiet: bool,
    pub recursive: bool,
}

#[derive(Default, Debug)]
pub struct ReplaceConfig {
    pub quiet: bool,
    pub nth: usize,
    pub replace_all: bool,
    pub fill_byte: u8,
    pub allow_length_change: bool,
}

#[derive(Default, Debug)]
pub struct InsertConfig {
    pub quiet: bool,
}

#[derive(Debug)]
pub struct OpenedFile<'a> {
    pub file: File,
    pub path: &'a Path,
}

#[derive(Debug)]
pub struct Match<'a> {
    pub opened_file: &'a mut OpenedFile<'a>,
    pub pattern: &'a [u8],
    pub offset: u64,
}

#[derive(Debug)]
pub struct OpenFiles<'a, T: AsRef<Path> + 'a> {
    pub files: &'a [T],
    pub nth: usize,
}

impl<'a, T: AsRef<Path> + 'a> Iterator for OpenFiles<'a, T> {
    type Item = OpenedFile<'a>;
    fn next(&mut self) -> Option<OpenedFile<'a>> {
        match self.files.get(self.nth) {
            Some(filename) => {
                self.nth += 1;
                open_file(filename.as_ref()).ok()
            },
            None => None,
        }
    }
}

impl<'a> Iterator for Match<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let pattern_len = self.pattern.len();
        // This is a slow O(n^2) way to do it. Obviously we can be smarter about it,
        // using a proper string-searching algorithm
        let mut buf = vec![0u8; pattern_len];
        self.opened_file.file.seek(SeekFrom::Start(self.offset)).unwrap();
        while let Ok(()) = self.opened_file.file.read_exact(&mut buf) {
            self.offset += 1;
            if buf == self.pattern {
                return Some(self.offset-1);
            }
            self.opened_file.file.seek(SeekFrom::Start(self.offset)).unwrap();
        }
        None
    }
}

fn find_matches<'a>(
    opened_file: &'a mut OpenedFile<'a>,
    pattern: &'a [u8]
) -> Match<'a> {
    Match::<'a>{ opened_file, pattern, offset: 0 }
}

fn open_file<'a>(filename: &'a Path) -> Result<OpenedFile<'a>, io::Error> {
    match File::open(&filename) {
        Ok(f) => Ok(OpenedFile {file: f, path: filename}),
        Err(e) => {
            eprintln!("Can't open {} bacause of error: {}", filename.display(), e);
            Err(e)
        }
    }
}

fn open_files<T: AsRef<Path>>(filenames: &[T]) -> Result<Vec<OpenedFile>, io::Error> {
    let mut ret = Vec::new();
    for filename in filenames {
        let file = open_file(filename.as_ref())?;
        ret.push(file);
    }
    Ok(ret)
}

fn open_files2<'a, T: AsRef<Path>>(filenames: &'a [T]) -> OpenFiles<'a, T> {
    OpenFiles { files: filenames, nth: 0 }
}

/// Open a directory recursively, getting all the files in the
/// directory and its subdirectories. Doesn't work with symlinks. We
/// make an assumption that the dir argument is a directory.
fn open_recursively(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut ret = Vec::new();
    for entry in dir.read_dir()? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            ret.push(entry.path());
        } else if metadata.is_dir() {
            ret.append(&mut open_recursively(&entry.path())?);
        }
    }
    
    Ok(ret)
}

fn open_all_directories<T: AsRef<Path>>(paths: &[T]) -> Result<Vec<PathBuf>, io::Error> {
    let mut ret = Vec::new();
    for path in paths {
        if path.as_ref().is_dir() {
            ret.append(&mut open_recursively(path.as_ref())?);
        } else if path.as_ref().is_file() {
            ret.push(path.as_ref().to_path_buf());
        }
    }
    
    Ok(ret)
}

pub fn grep_command<T: AsRef<Path>>(
    pattern: &[u8],
    filenames: &[T],
    grep_config: &GrepConfig,
) -> Result<(), io::Error> {
    // Handle directories as paths
    let files;
    let paths: Vec::<&Path>;
    if grep_config.recursive {
        files = open_all_directories(filenames)?;
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

pub fn grep<T: AsRef<Path>>(
    pattern: &[u8],
    filenames: &[T],
) -> Result<Vec<(PathBuf, Vec<u64>)>, io::Error> {
    let mut ret = Vec::new();
    for mut file in open_files2(filenames) {
        ret.push((PathBuf::from(file.path), Vec::new()));
        let found_matches: Vec<_> = find_matches(&mut file, pattern).collect();
        for &offset in found_matches.iter() {
            ret.last_mut().unwrap().1.push(offset);
        }
    }
    
    Ok(ret)
}

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

pub fn replace(
    to_replace: &[u8],
    replace_with: &[u8],
    input_filename: &Path,
    output_filename: &Path,
    replace_config: &ReplaceConfig,
) -> Result<usize, io::Error> {
    let mut input_file = open_file(input_filename)?;
    
    let mut matches_iter = find_matches(&mut input_file, to_replace);
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

    let to_fill = if replace_config.allow_length_change {
        0
    } else {
        to_replace.len() - replace_with.len()
    };
    let mut input_file = File::open(input_filename)?;
    let mut output_file = File::create(output_filename)?;
    let mut last_offset = 0;
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
    let mut buf = String::new();
    input_file.read_to_string(&mut buf)?;
    output_file.write(buf.as_bytes())?;
    
    Ok(found_matches.len())
}

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

pub fn insert(
    to_insert: &[u8],
    offset: usize,
    input_filename: &Path,
    output_filename: &Path,
) -> Result<(), io::Error> {
    let mut input_file = open_file(input_filename)?;
    let mut output_file = File::create(output_filename)?;

    // This will crash if there's not enough RAM but it's good enough for now.
    // Why Rust doesn't have sendfile(2)?!
    // A simple solution would be to read the file in chunks.
    let mut buf = vec![0u8; offset];
    input_file.file.read_exact(&mut buf)?;
    output_file.write(&buf)?;
    output_file.write(&to_insert)?;
    let mut buf = String::new();
    input_file.file.read_to_string(&mut buf)?;
    output_file.write(buf.as_bytes())?;

    Ok(())
}
