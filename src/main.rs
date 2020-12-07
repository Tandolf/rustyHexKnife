use std::{error::Error, fs::{self, File, OpenOptions}, io::{BufReader, Read, Write}, io::SeekFrom, path::Path, io::Seek};

use clap::{App, Arg, crate_authors, crate_description, crate_name, crate_version};
use fs::metadata;

const MB_SIZE: usize = 0x100000;
const BACKUP_FOLDER: &str = "./header_backups/";

fn visit_dirs(source: &str, dir: &Path, bytes: u64) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dirs(&source, &path, bytes)?;
        } else {
            do_copy(source, &entry.path())?;
            truncate(&entry.path(), bytes)?;
        }
    }
    Ok(())
}

fn truncate(file_path: &Path, bytes: u64) -> Result<(), Box<dyn Error>> {

    let f = OpenOptions::new()
    .write(true)
    .open(&file_path)?;

    let metadata = f.metadata().unwrap();

    let new_length = metadata.len() - bytes;

    f.set_len(new_length)?;

    println!("Truncated {} bytes from file: {:?}", bytes, file_path.file_name().unwrap());

    Ok(())
}

fn do_copy(source: &str, dest: &Path) -> Result<(), Box<dyn Error>> {

    // Open destination file
    let mut destination_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&dest)?;
 
    // Create buffer and read the header from destination file
    let mut buffer = [0; MB_SIZE];
    destination_file.read_exact(&mut buffer)?;
    
    // create the backup file
    let backup_filename = dest.file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string() + ".header.backup";
    
    let backup_filename = Path::new(BACKUP_FOLDER)
        .join(&backup_filename);
    let mut backup_file = File::create(backup_filename)?;
    
    // Write to the backup file
    backup_file.write_all(&buffer)?;
    backup_file.flush()?;
    
    // Open source file
    let source_file = File::open(&source)?;

    //create buffer and read from source
    let mut source_file_buffer = BufReader::new(&source_file);
    source_file_buffer.read_exact(&mut buffer)?;

    // Write buffer to destination file
    destination_file.seek(SeekFrom::Start(0))?;
    destination_file.write_all(&buffer)?;
    destination_file.flush()?;

    println!("{}{:?}", "Header replaced for: ", dest.file_name().unwrap());

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>>{

    let matches = App::new(crate_name!())
                            .version(crate_version!())
                            .author(crate_authors!())
                            .about(crate_description!())
                            .arg(Arg::with_name("source")
                                    .short("s")
                                    .value_name("FILE")
                                    .required(true)
                                    .takes_value(true)
                                    .help("Set the source file")
                            ).arg(Arg::with_name("dest")
                                    .short("d")
                                    .value_name("FILE")
                                    .required(true)
                                    .takes_value(true)
                                    .help("Set the destination file or directory")
                            ).arg(Arg::with_name("bytes")
                                .short("b")
                                .value_name("NUMBER")
                                .required(true)
                                .takes_value(true)
                                .help("Set number of bytes to be truncated")
                    ).get_matches();
    
    let source_filename = matches.value_of("source").unwrap().trim();
    let dest_filename = matches.value_of("dest").unwrap().trim();
    let bytes = matches.value_of("bytes").unwrap().trim().parse::<u64>().unwrap();

    let md = metadata(&dest_filename).unwrap();

    if !Path::new(BACKUP_FOLDER).exists() {
        fs::create_dir(BACKUP_FOLDER)?;
    }

    let dest_path = Path::new(dest_filename);
    if md.is_file() {
        do_copy(&source_filename, &dest_path)?;
        truncate(&dest_path, bytes)?;
    }

    if md.is_dir() {
        visit_dirs(source_filename, dest_path, bytes)?;
    }

    Ok(())
}
