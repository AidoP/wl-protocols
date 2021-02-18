use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf}
};

use clap::{Arg, App};

mod protocol;
use protocol::Protocol;

#[macro_export]
macro_rules! error {
    ($message:expr $(, $($arg:tt)*)?) => {{
        eprintln!($message, $($($arg)*)?);
        return;
    }};
}
#[macro_export]
macro_rules! fatal {
    ($result:expr, $message:expr $(,$arg:tt)*) => {
        match $result {
            Ok(x) => x,
            Err(e) => error!($message $(, $arg)*, error = e)
        }
    };
    ($result:expr => $err:expr) => {
        match $result {
            Ok(x) => x,
            Err(e) => return Err($err)
        }
    };
}

fn main() {
    let matches = App::new("wl-protocol")
        .version("0.0.0")
        .about("Translate XML Wayland protocol specifications into TOML")
        .author("AidoP, https://github.com/AidoP")
        .arg(
            Arg::with_name("INPUT")
                .required(true)
                .help("Path to XML protocol(s)")
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .help("The file to output the TOML protocol to")
                .takes_value(true)
                .value_name("OUTPUT")
        )
        .arg(
            Arg::with_name("recursive")
                .short("r")
                .long("recursive")
                .requires("output")
                .help("Convert a directory")
        )
        .get_matches();
    
    if matches.is_present("recursive") {
        fn search_dir(path: &Path, top_dir: &Path, out_dir: &Path) {
            let entries = fatal!(fs::read_dir(path), "Unable to open {:?} for searching. {error}", path);
            for entry in entries {
                let entry = fatal!(entry, "Failed to read directory entries. {error}");
                let path = &entry.path();
                let file_type = fatal!(entry.file_type(), "Unable to determine file type of {:?}. {error}", path);
                if file_type.is_dir() {
                    search_dir(&path, top_dir, out_dir)
                } else if file_type.is_symlink() {
                    eprintln!("Ignoring symlink")
                } else if file_type.is_file() && path.extension().map(|e| e == "xml").unwrap_or(false) {
                    let relative = fatal!(path.strip_prefix(top_dir), "Unable to calculate the destination path for file {:?}. {error}", path);
                    let mut destination = PathBuf::from(out_dir);
                    destination.push(relative);
                    destination.set_extension("toml");

                    let destination_parent = fatal!(destination.parent().ok_or(&destination), "Subdirectory {:?} is invalid as it is not a subdirectory...");
                    fatal!(fs::create_dir_all(destination_parent), "Unable to create subdirectory {:?}. {error}", destination_parent);
                    
                    let mut input_file = fatal!(File::open(path), "Unable to open path {:?}. {error}", path);
                    let mut xml = String::new();
                    fatal!(input_file.read_to_string(&mut xml), "Unable to read file {:?}, {error}", path);
                    let protocol = fatal!(Protocol::load_xml(&xml), "Unable to parse file {:?}. {error}", path);

                    let mut output_file = fatal!(File::create(&destination), "Unable to open path {:?}. {error}", destination);
                    let string = fatal!(protocol.to_string(), "Unable to serialize the protocol as TOML. {error}");
                    fatal!(output_file.write_all(string.as_bytes()), "Unable to write out TOML. {error}")
                }
            }
        }
        let path = Path::new(matches.value_of_os("INPUT").unwrap());
        search_dir(path, path, Path::new(matches.value_of_os("output").unwrap()))
    } else {
        let path = matches.value_of_os("INPUT").unwrap();
        let mut input_file = fatal!(File::open(path), "Unable to open path {:?}. {error}", path);
        let mut xml = String::new();
        fatal!(input_file.read_to_string(&mut xml), "Unable to read file {:?}, {error}", path);
        let protocol = fatal!(Protocol::load_xml(&xml), "Unable to parse file {:?}. {error}", path);
        if let Some(path) = matches.value_of_os("output") {
            let mut output_file = fatal!(File::create(path), "Unable to open path {:?}. {error}", path);
            let string = fatal!(protocol.to_string(), "Unable to serialize the protocol as TOML. {error}");
            fatal!(output_file.write_all(string.as_bytes()), "Unable to write out TOML. {error}")
        } else {
            let string = fatal!(protocol.to_string(), "Unable to serialize the protocol as TOML. {error}");
            println!("{}", string);
        }
    }
}

#[test]
fn test_all() {
    let in_path = "/usr/share/wayland-protocols/";
    let out_path = "protocol/";

    fn compare(xml_path: &Path, toml_path: &Path) {
        let mut input_file = fatal!(File::open(xml_path), "Unable to open path {:?}. {error}", xml_path);
        let mut xml = String::new();
        fatal!(input_file.read_to_string(&mut xml), "Unable to read file {:?}, {error}", xml_path);
        let xml_protocol = fatal!(Protocol::load_xml(&xml), "Unable to parse file {:?}. {error}", xml_path);

        let mut output_file = fatal!(File::open(&toml_path), "Unable to open path {:?}. {error}", toml_path);
        let mut toml = String::new();
        fatal!(output_file.read_to_string(&mut toml), "Unable to read file {:?}, {error}", toml_path);
        let toml_protocol = fatal!(Protocol::load_toml(&toml), "Unable to parse file {:?}. {error}", toml_path);

        println!("Comparing {:?} and {:?}", xml_path, toml_path);
        assert_eq!(xml_protocol, toml_protocol);
    }

    fn search_dir(path: &Path, top_dir: &Path, out_dir: &Path) {
        let entries = fatal!(fs::read_dir(path), "Unable to open {:?} for searching. {error}", path);
        for entry in entries {
            let entry = fatal!(entry, "Failed to read directory entries. {error}");
            let path = &entry.path();
            let file_type = fatal!(entry.file_type(), "Unable to determine file type of {:?}. {error}", path);
            if file_type.is_dir() {
                search_dir(&path, top_dir, out_dir)
            } else if file_type.is_symlink() {
                eprintln!("Ignoring symlink")
            } else if file_type.is_file() && path.extension().map(|e| e == "xml").unwrap_or(false) {
                let relative = fatal!(path.strip_prefix(top_dir), "Unable to calculate the destination path for file {:?}. {error}", path);
                let mut destination = PathBuf::from(out_dir);
                destination.push(relative);
                destination.set_extension("toml");
                
                compare(path, destination.as_path());
            }
        }
    }
    compare(Path::new("/usr/share/wayland/wayland.xml"), Path::new("protocol/wayland.toml"));
    let path = Path::new(in_path);
    search_dir(path, path, Path::new(out_path))
}