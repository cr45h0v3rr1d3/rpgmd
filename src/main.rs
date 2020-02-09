use serde_json::Value;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
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

    for rpg_file in rpg_files {
        decrypt_file(&key, &rpg_file, is_inplace, output_folder);
    }
}

fn decrypt_file(key: &Vec<u8>, input_path: &PathBuf, is_inplace: bool, output_folder: &str) {
    let mut output_file = get_output_file(input_path, is_inplace, output_folder);

    let mut input_buffer = vec![];

    let mut input_file = File::open(input_path).expect("Failed to open input rpg file");
    input_file
        .read_to_end(&mut input_buffer)
        .expect("Failed to read input rpg file");

    // ignore fake header
    //
    let offset = 16;
    for i in 0..16 {
        input_buffer[offset + i] = key[i] ^ input_buffer[offset + i];
    }

    output_file
        .write_all(&input_buffer[offset..])
        .expect("Failed to write to output file");
}

fn get_output_file(input_path: &PathBuf, is_inplace: bool, output_folder: &str) -> File {
    let new_file_extension = match input_path.extension().unwrap().to_str().unwrap() {
        "rpgmvp" => "png",
        "rpgmvm" => "m4a",
        "rpgmvo" => "ogg",
        _ => "png",
    };

    let parent_path: PathBuf;
    if is_inplace {
        parent_path = match input_path.parent() {
            Some(p) => PathBuf::from(p),
            None => Path::new(".").to_path_buf(),
        };
    } else {
        parent_path = Path::new(output_folder).to_path_buf();
    }

    let mut new_file_name: String = parent_path.to_str().unwrap().to_string();
    new_file_name.push_str("/");
    new_file_name.push_str(input_path.file_stem().unwrap().to_str().unwrap());
    new_file_name.push_str(".");
    new_file_name.push_str(new_file_extension);

    println!("{}", new_file_name);

    let new_file = Path::new(&new_file_name);

    File::create(new_file).expect("Failed to open output rpg file")
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
        0 | 1 => ("./", true, ""),
        2 => (&args[1], true, ""),
        3 => {
            std::fs::create_dir_all(&args[2]).expect("Failed to create output directory");
            (&args[1], false, &args[2])
        }
        _ => {
            panic!("Usage: rpgmd <game_folder> <output_folder:optional>");
        }
    }
}
