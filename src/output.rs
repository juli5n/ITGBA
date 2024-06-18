use crate::input_data_representation_types::*;
use crate::read_input::*;

use crate::helper::*;

use derive_more::{Deref,DerefMut};
use std::path::{PathBuf, Path};
use std::collections::HashMap;

pub struct Output_info_for_a_single_file { 
    pub content_string: String,
    pub specified_path: std::path::PathBuf,
}


#[derive(Deref,DerefMut)]
pub struct Output(pub Vec<Output_info_for_a_single_file>);

impl Output {
    pub fn write_to_disk(self, output_directory: &Option<PathBuf>, working_directory: &PathBuf, mimic_relative_paths_to_input_directory: bool) {
        std::env::set_current_dir(working_directory).unwrap();

        for output_info_for_a_single_file in self.0 {

            // Find relative path to output directory
            let mut relative_path_to_output_directory = output_info_for_a_single_file.specified_path;
            if relative_path_to_output_directory.is_absolute() || (!mimic_relative_paths_to_input_directory){
                relative_path_to_output_directory = PathBuf::from(relative_path_to_output_directory.file_name().unwrap());
            }
            // Add c extension
            relative_path_to_output_directory.set_extension("c");

        

            // Prepend output directory to the specified path if an output directory was specified
            let path_adjusted_for_output_directory = if let Some(output_directory) = output_directory.as_ref() {
                output_directory.join(relative_path_to_output_directory)
            } else {
                relative_path_to_output_directory 
            };

            // create directory and parent directories if they don't exist
            if let Some(parent) = path_adjusted_for_output_directory.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            println!("writing to: {}", path_adjusted_for_output_directory.to_str().unwrap());
            std::fs::write(path_adjusted_for_output_directory, output_info_for_a_single_file.content_string).unwrap();
        }
    }
}

pub fn create_output_info_for_tilemap_path(
    tilemap_image_path: &Path, 
    reference_color_palette: &ColorPalette,
    tile_search_map: &HashMap<TileColorArray, TileSearchmapValue> ,
    allow_attributes_and_generate_attribute_array: bool, 
    use_hex_notation: bool,
) -> Output_info_for_a_single_file {
    let (index_array, attribute_array): (TileIndexArray, Option<AttributeByteArray>) = index_and_attribute_array_from_tilemap_image_path(tilemap_image_path, &reference_color_palette, &tile_search_map, allow_attributes_and_generate_attribute_array);
    let mut output_info = Output_info_for_a_single_file::new(tilemap_image_path);

    output_info.write_tile_index_array(&index_array, use_hex_notation);

    if let Some(attribute_byte_array) = attribute_array {
        output_info.write_attribute_byte_array(&attribute_byte_array, use_hex_notation);
    }

    return output_info;
}


impl Output_info_for_a_single_file {
    pub fn new<T>(specified_path: T) -> Self where T:Into<PathBuf>{

        let mut res = Self {
            content_string: String::new(),
            specified_path: specified_path.into(),
        };
        res.write_header();
        return res;
    }
    fn c_file_name_osstring(&self) -> std::ffi::OsString {
        let mut res = self.specified_path.file_stem().unwrap().to_owned();
        res.push(".c");
        return res;
    }
    fn c_file_name_string(&self) -> String {
        return self.c_file_name_osstring().into_string().unwrap();
    }
    fn filename_without_extension_string(&self) -> String{
        self.specified_path.file_stem().unwrap().to_owned().into_string().unwrap()
    }
    /// Takes the specified path and modifies the extension
    fn relative_path_but_with_c_extension_osstring(&self) -> std::ffi::OsString {
        return self.relative_path_but_with_extension_osstring(".c");
    }
    fn relative_path_but_with_c_extension_pathbuf(&self) -> PathBuf {
        return PathBuf::from(self.relative_path_but_with_c_extension_osstring());
    }
    fn relative_path_but_with_c_extension_string(&self) -> String {
        return self.relative_path_but_with_c_extension_osstring().into_string().unwrap();
    }

    fn relative_path_but_with_extension_osstring(&self, extension: &str) -> std::ffi::OsString {
        let mut res = self.specified_path.file_stem().unwrap().to_owned();
        res.push(extension);
        return res;
    }

    pub fn write_header(&mut self) {
        self.content_string.push_str(
            format!(
                "// {} - Generated file by ITGBA \n",
                self.c_file_name_string()
            )
            .as_str(),
        );
    }

    pub fn write_attribute_byte_array(&mut self, attributes_array: &AttributeByteArray, use_hex_notation: bool) {
        self.content_string.push_str(
            format!(
                "const unsigned char {}_tilemap_attribute_array",
                self.filename_without_extension_string()
            )
            .as_str(),
        );
        self.write_tilemap_byte_array(&attributes_array.0, use_hex_notation);

    }
    pub fn write_tile_index_array(&mut self, index_array: &TileIndexArray, use_hex_notation: bool) {
        self.content_string.push_str(
            format!(
                "const unsigned char {}_tileindex__array",
                self.filename_without_extension_string()
            )
            .as_str(),
        );
        self.write_tilemap_byte_array(&index_array.0, use_hex_notation);

    }

    pub fn write_tileset(&mut self, tiledata_vec: &Vec<TileInfo>, use_hex_notation: bool) {


        // Write constants that give names to the indices of tiles that have an identifier name
        self.content_string.push_str(
            format!(
                "// Constants for easier tile indexing. These constants are generated using\n\
                // the file names of the individual tiles if they don't have an ignore prefix \"{}\"\n\
                // and are valid c identifiers.", crate::read_input::IGNORE_PREFIX).as_str()
        );
        for (tile_index, tile_info) in tiledata_vec.iter().enumerate() {
           if let Some(name) = tile_info.name.as_ref() {
                self.content_string.push_str(
                    format!("const size_t {}_tile_index = {};\n", name, tile_index).as_str()
                );
           }
        }
        self.content_string.push_str("\n");

        self.content_string.push_str(
            format!("const unsigned char {}[] = \n", self.c_file_name_string().as_str()).as_str()
        );

        for (tile_index, tile_info) in tiledata_vec.iter().enumerate() {
            
            self.content_string.push_str(format!("\n\t// Tile {}\n", tile_index).as_str());

            for y in 0..8 {
                if (y  % 2 == 0) {
                    self.content_string.push_str("\t");
                }
                // write 2 bytes corresponding to the line x
                let mut first_byte: u8 = 0; // stores the least significant bits of the palette indices
                let mut second_byte: u8 = 0; // stores the highest significant bits " 

                for x in 0..8 {
                    let palette_index = tile_info.color_array.get(x,y);
                    first_byte += ((palette_index & 1u8) << 7) >> x;
                    second_byte += (((palette_index & 2u8) >> 1) << 7) >> x;
                }

                self.content_string
                    .push_str(match use_hex_notation {
                        true => format!("{:#04x}, {:#04x}, ", first_byte, second_byte),
                        false => format!("{:#010b}, {:#010b}, ", first_byte, second_byte),
            }.as_str());

                if ((y + 1) % 2 == 0) {
                    self.content_string.push_str(format!(" // Line {}-{}\n", y-1, y).as_str());
                }
            }
        }


    }
    pub fn write_tilemap_byte_array(&mut self, byte_array: &TilemapByteArray, use_hex_notation: bool) {

        self.content_string.push_str(
            format!("const unsigned char {}[] = \n", self.c_file_name_string().as_str()).as_str()
        );

        let mut byte_index = 0;
        for y in 0..32{
            for x in 0..32 { 
                
                let byte: u8 = byte_array.get(x,y);

                self.content_string
                    .push_str(match use_hex_notation {
                        true => format!("{:#04x}, ", byte),
                        false => format!("{:#010b}, ", byte),
                    }.as_str());

                if ((byte_index + 1) % 8 == 0) {
                    self.content_string.push_str("\n");
                }

                byte_index+=1;
            }
        }

    }

}

