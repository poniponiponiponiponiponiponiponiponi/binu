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
    pub recursive: bool,
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

pub fn grep_command<T: AsRef<Path>>(
    pattern: &[u8],
    filenames: &[T],
    grep_config: &GrepConfig,
) -> Result<(), io::Error> {
    let results = grep(pattern, filenames)?;
    let is_empty: bool = results.iter().all(|e| e.1.is_empty());
    if is_empty {
        if !grep_config.quiet {
            println!("Nothing found");
        }
        return Ok(());
    }
    
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
    let mut files = open_files(filenames)?;
    for file in files.iter_mut() {
        ret.push((PathBuf::from(file.path), Vec::new()));
        let found_matches: Vec<_> = find_matches(file, pattern).collect();
        for &offset in found_matches.iter() {
            ret.last_mut().unwrap().1.push(offset);
        }
    }
    
    Ok(ret)
}

pub fn replace_command(
    replace: &[u8],
    replace_with: &[u8],
    input_filename: &Path,
    output_filename: &Path,
    replace_config: &ReplaceConfig,
) -> Result<(), io::Error> {
    Ok(())
}

pub fn replace(
    replace: &[u8],
    replace_with: &[u8],
    input_filename: &Path,
    output_filename: &Path,
    replace_config: &ReplaceConfig,
) -> Result<(), io::Error> {
    let file = open_file(input_filename)?;
    Ok(())
}

pub fn insert_command(
    to_insert: &[u8],
    offset: usize,
    input_filename: &Path,
    output_filename: &Path,
    insert_config: &InsertConfig
) -> Result<(), io::Error> {
    insert(to_insert, offset, input_filename, output_filename, insert_config)?;
    Ok(())
}

pub fn insert(
    to_insert: &[u8],
    offset: usize,
    input_filename: &Path,
    output_filename: &Path,
    insert_config: &InsertConfig
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

    if !insert_config.quiet {
        println!("Inserting was sucessful");
    }
    
    Ok(())
}
