pub use std::{convert::TryInto, fs::File, io::Write, path::Path};

mod binary_io;
mod case_sensitivity;
mod dat_section;
mod graphics_set;
mod main_dat;
mod parser;
mod planar_bmp;

use dat_section::DatSection;
use graphics_set::ExtractOptions;

fn cmd_extract_graphics_set(graphics_set: usize) {
    let terrain_filenames = format!("set{}_terrain#.bmp", graphics_set);
    let terrain_mask_filenames = format!("set{}_terrain#_mask.bmp", graphics_set);
    let object_filenames = format!("set{}_obj#.bmp", graphics_set);
    let object_mask_filenames = format!("set{}_obj#_mask.bmp", graphics_set);
    let options = ExtractOptions {
        terrain_filename_pattern: &terrain_filenames,
        terrain_mask_filename_pattern: Some(&terrain_mask_filenames),
        object_filename_pattern: &object_filenames,
        object_mask_filename_pattern: Some(&object_mask_filenames),
        ega_mode: false,
    };

    let script_filename = format!("theme{}.txt", graphics_set);
    let script_file = std::fs::File::create(script_filename).unwrap();
    let mut script_writer = std::io::BufWriter::new(&script_file);

    let ground_filename = format!("ground{}o.dat", graphics_set);
    let ground_path = case_sensitivity::find_file_in_current_dir(ground_filename.as_str()).unwrap();
    let mut ground_header_file = match File::open(&ground_path) {
        Err(err) => panic!("Error opening ground header file: {}|", err),
        Ok(file) => file,
    };

    let data_filename = format!("vgagr{}.dat", graphics_set);
    let path = case_sensitivity::find_file_in_current_dir(data_filename.as_str()).unwrap();
    let path_name = path.display();
    let mut image = match File::open(&path) {
        Err(err) => panic!("Error opening lemmings file {}: {}", path_name, err),
        Ok(file) => file,
    };

    // Write the header for the graphics set script.
    write!(script_writer, "HeaderFile \"{}\"\n", ground_filename).unwrap();
    write!(script_writer, "DataFile \"{}\"\n\n", data_filename).unwrap();

    graphics_set::extract_graphics_set(
        &mut script_writer,
        &mut ground_header_file,
        &mut image,
        &options,
    );
}

fn cmd_create_graphics_set(filename: &str) {
    let script_data = std::fs::read_to_string(filename).unwrap();
    let mut lexer = parser::Lexer::from_str(script_data.as_str());

    graphics_set::create_graphics_set(&mut lexer)
}

fn cmd_extract_main_dat(xmas_mode: bool) {
    let path = case_sensitivity::find_file_in_current_dir("main.dat").unwrap();

    let mut main_dat_file = match File::open(&path) {
        Err(err) => panic!("Error opening main.dat: {}", err),
        Ok(file) => file,
    };

    main_dat::extract_main_dat(&mut main_dat_file, xmas_mode);
}

fn cmd_create_main_dat() {
    main_dat::create_main_dat();
}

/// Splits and decompresses [name].dat file into its consituant sections,
/// each named [name].000, [name].001, etc.
fn extract_dat(name: &std::string::String) {
    let dat_filename = format!("{}.dat", name);
    let dat_path = case_sensitivity::find_file_in_current_dir(&dat_filename).unwrap();

    let mut data = match File::open(&dat_path) {
        Err(err) => panic!("Error opening {}: {}", dat_filename, err),
        Ok(file) => file,
    };

    let mut section_num = 0;
    loop {
        let section_res = DatSection::from_file(&mut data);
        match section_res {
            Err(__) => break,
            Ok(mut header) => {
                let section_data = header.decompress();
                let outfile_name = format!("{}.{:03}", name, section_num);
                let out_path = Path::new(outfile_name.as_str());
                let mut output_file = File::create(out_path).unwrap();
                output_file.write_all(section_data.as_slice()).unwrap();
            }
        }
        section_num += 1;
    }
}

fn create_dat(name: &std::string::String) {
    let dat_filename = format!("{}.dat", name);
    let dat_path = Path::new(&dat_filename);

    let mut data = match File::create(&dat_path) {
        Err(err) => panic!("Error opening {}: {}", dat_filename, err),
        Ok(file) => file,
    };

    let mut section_num = 0;
    loop {
        let section_file_name = format!("{}.{:03}", name, section_num);
        let section_path = Path::new(section_file_name.as_str());
        let section_uncomp_data = std::fs::read(section_path);
        match section_uncomp_data {
            Err(__) => break,
            Ok(uncomp_data) => {
                let section = DatSection::from_data(uncomp_data.as_slice(), uncomp_data.len());
                section.write(&mut data).unwrap();
            }
        }
        section_num += 1;
    }
}

fn show_usage() {
    println!("Usage:");
    println!("\tmodlem extract-set <n>");
    println!("\t\tExtracts graphics set <n>");
    println!("\tmodlem create-set <script-name>");
    println!("\t\tCreates a graphics set from a script file.");
    println!("\tmodlem extract-main");
    println!("\t\tExtracts main.dat into its constituent files.");
    println!("\tmodlem create-main");
    println!("\t\tCreates a main.dat from bitmaps in the current directory.");
    println!("\tmodlem extract-dat <name>");
    println!("\t\tDecompresses <name>.dat into <name>.000, <name>.001, etc.");
    println!("\tmodlem create-dat <name>");
    println!("\t\tCompressed <name>.000, <name>.001, etc. into <name>.dat");
}

fn main() {
    let args: Vec<std::string::String> = std::env::args().collect(); /* Skip the application name. */

    if args.len() < 2 {
        show_usage();
        return;
    }

    let command_name = &args[1];

    match command_name.as_str() {
        "extract-set" => {
            let set_num = args[2].parse::<usize>().unwrap();
            println!("Extracting graphics set {}…", set_num);
            cmd_extract_graphics_set(set_num);
        }
        "create-set" => {
            let script_name = &args[2];
            println!("Creating graphics set from \"{}\"", script_name);
            cmd_create_graphics_set(script_name);
        }
        "extract-main" => {
            let mut xmas_mode = false;
            let mut arg_iter = args.iter().skip(2);
            while let Some(arg) = arg_iter.next() {
                match arg.as_str() {
                    "--xmas" | "--christmas" => xmas_mode = true,
                    _ => panic!("Unknown argument \"{}\"", arg),
                }
            }
            cmd_extract_main_dat(xmas_mode);
        }
        "create-main" => {
            cmd_create_main_dat();
        }
        "extract-dat" => {
            let dat_name = &args[2];
            println!("Extracting {}.dat…", dat_name);
            extract_dat(&dat_name);
        }
        "create-dat" => {
            let dat_name = &args[2];
            println!("Create {}.dat…", dat_name);
            create_dat(&dat_name);
        }
        invalid_cmd => {
            panic!("Unknown command \"{}\"", invalid_cmd);
        }
    }
}
