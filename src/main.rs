#![allow(warnings)]
#![feature(path_file_prefix, generic_const_exprs)]
use image::{DynamicImage, Rgb};
use std::path::{
    Path,
    PathBuf
};
use std::ops::Deref;
use std::thread::current;
use std::vec::Vec;
use std::{collections::{HashMap, LinkedList}, env};
use clap::Parser;
use derive_more::{Deref, DerefMut};

mod output;
mod read_input;
mod cli_parser;
mod input_data_representation_types;
mod helper;

use output::*;
use cli_parser::*;
use read_input::*;
use input_data_representation_types::*;
use helper::*;




fn main() {
    // Parse command line arguments
    let mut parse_result = Cli_parser::parse();

    let initial_working_directory = std::env::current_dir().unwrap(); 

    // Change working directory during reading to input_directory
    if let Some(input_directory) = parse_result.input_directory.as_ref() {
        std::env::set_current_dir(input_directory).expect(format!("Couldn't change working directory to the specified input directory: \"{}\"", input_directory.to_str().unwrap()).as_str())
    }
    // Setup output vector that contains info entries for each output file
    let mut output: Output = Output(Vec::new());

    // Reference tileset output file info entry
    // TODO If the single file file flag is set, this will store the output info for the only output file
    let mut reference_tileset_output_info = Output_info_for_a_single_file::new(match parse_result.reference_tileset_path.is_dir() {
        true => {
            let mut res = parse_result.reference_tileset_path.clone();
            res.set_file_name("tileset");
            res 
        },
        false => parse_result.reference_tileset_path.clone(),
    });
    

    // Process reference tileset file
    let (tile_info_vec, color_palette): (Vec<TileInfo>, ColorPalette) = read_tileset_info_from_path(&parse_result.reference_tileset_path);

    // Write the retrieved information from the tileset to output info
    reference_tileset_output_info.write_tileset(&tile_info_vec, parse_result.use_hex);

    let tile_search_map = tile_searchmap_from_tiledata_vec(tile_info_vec);

    // Process tilemap arguments
    for (tilemap_image_path, allow_attributes) in parse_result.map_file_paths.iter().map(|x| (x, false)).chain( parse_result.map_with_attributes_file_paths.iter().map(|x| (x,true)) ) {
        output.push(
            create_output_info_for_tilemap_path(tilemap_image_path, &color_palette, &tile_search_map, allow_attributes, parse_result.use_hex)
        );
    }


    output.push(reference_tileset_output_info);

    output.write_to_disk(&parse_result.output_directory, &initial_working_directory, parse_result.mimic_relative_paths_to_input_directory);

}




