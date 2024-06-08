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


    // Setup output vector that contains info entries for each output file
    let mut output: Output = Output(Vec::new());

    // Reference tileset output file info entry
    // TODO If the single file file flag is set, this will store the output info for the only output file
    let mut reference_tileset_output_info = Output_info_for_a_single_file {
        content_string: String::new(),
        path_relative_to_output_directory: parse_result.reference_tileset_path.clone()
    };

    // Process reference tileset file
    let (tiledata_vec, color_palette): (Vec<Tiledata>, ColorPalette) = read_tileset_info_from_path(&parse_result.reference_tileset_path);

    // Write the retrieved information from the tileset to output info
    write_tileset_to_output_file_info(&tiledata_vec, &mut reference_tileset_output_info, parse_result.use_hex);

    // Process tilemap arguments
    for tile_map_path in parse_result.map_file_paths {
        //TODO
    }


    output.push(reference_tileset_output_info);

    write_output_to_disk(output, &parse_result.output_directory);

}




