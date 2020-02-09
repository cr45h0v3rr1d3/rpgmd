use serde_json::Value;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

extern crate hex;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (input_folder, is_inplace, output_folder) = parse_config(&args);

    let key = find_system_file(input_folder.to_string()).unwrap();

    let mut rpg_files: Vec<PathBuf> = Vec::new();
    find_files(Path::new(input_folder), &mut rpg_files);
}

fn find_key(system_file_path: &PathBuf) -> Option<Vec<u8>> {
    println!("Checking for key in {}", system_file_path.display());
    let file = File::open(system_file_path).expect("Failed to open system json file");
    let reader = BufReader::new(file);

    let system_json: Value =
        serde_json::from_reader(reader).expect("Could not parse system json file");

    let key_str = parse_system_json(&system_json)?;

    println!("Found key {}", key_str);
    Some(hex::decode(key_str).expect("Failed to decode encryption key"))
}

fn parse_system_json(system_json: &Value) -> Option<&str> {
    return system_json.as_object()?.get("encryptionKey")?.as_str();
}

fn find_files(input_folder: &Path, mut rpgmv_files: &mut Vec<PathBuf>) {
    let paths = fs::read_dir(input_folder).unwrap();
    for file_in_dir in paths {
        let path = file_in_dir.unwrap().path();
        if path.is_dir() {
            find_files(path.as_path(), &mut rpgmv_files);
        } else {
            let file_name = path.file_name();
            if file_name == Some(OsStr::new("System.json"))
                || file_name == Some(OsStr::new("system.json"))
            {
            } else if path.extension() == Some(OsStr::new("rpgmvp")) {
                rpgmv_files.push(path);
            } else if path.extension() == Some(OsStr::new("rpgmvm")) {
                rpgmv_files.push(path);
            } else if path.extension() == Some(OsStr::new("rpgmvo")) {
                rpgmv_files.push(path);
            }
        }
    }
}

fn find_system_file(input_folder_str: String) -> Option<Vec<u8>> {
    let input_folder: String = input_folder_str;
    loop {
        let paths = fs::read_dir(input_folder.to_string()).unwrap();
        for file_in_dir in paths {
            let path = file_in_dir.unwrap().path();
            if path.is_dir() {
                let optional = find_system_file(path.as_path().to_str().unwrap().to_string());
                if optional.is_some() {
                    return optional;
                }
            } else {
                let file_name = path.file_name();
                if file_name == Some(OsStr::new("System.json"))
                    || file_name == Some(OsStr::new("system.json"))
                {
                    let key_opt = find_key(&path);
                    if key_opt.is_some() {
                        return key_opt;
                    }
                }
            }
        }

        return None;
    }
}

fn parse_config(args: &[String]) -> (&str, bool, &str) {
    match args.len() {
        0 | 1 => ("./", false, ""),
        2 => (&args[1], false, ""),
        3 => (&args[1], true, &args[2]),
        _ => {
            panic!("Usage: rpgmd <game_folder> <output_folder:optional>");
        }
    }
}
