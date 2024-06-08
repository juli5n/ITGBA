use crate::input_data_representation_types::*;
use derive_more::{Deref,DerefMut};
use std::path::PathBuf;

pub struct Output_info_for_a_single_file { 
    pub content_string: String,
    pub path_relative_to_output_directory: std::path::PathBuf,
}


#[derive(Deref,DerefMut)]
pub struct Output(pub Vec<Output_info_for_a_single_file>);

pub fn write_output_to_disk(output: Output, output_directory: &Option<PathBuf>) {
    for output_info_for_a_single_file in output.0 {
        // Prepend output directory to the specified path if an output directory was specified
        let path_adjusted_for_output_directory = if let Some(output_directory) = output_directory.as_ref() {
            output_directory.join(output_info_for_a_single_file.path_relative_to_output_directory)
        } else {
            output_info_for_a_single_file.path_relative_to_output_directory
        };

        // create directory and parent directories if they don't exist
        std::fs::create_dir_all(path_adjusted_for_output_directory.clone()).unwrap();
        std::fs::write(path_adjusted_for_output_directory, output_info_for_a_single_file.content_string);
    }
}
impl Output_info_for_a_single_file {
    fn c_file_name_osstring(&self) -> std::ffi::OsString {
        let mut res = self.path_relative_to_output_directory.file_stem().unwrap().to_owned();
        res.push(".c");
        return res;
    }
    fn c_file_name_string(&self) -> String {
        return self.c_file_name_osstring().into_string().unwrap();
    }
    /// Takes the specified path and modifies the extension
    fn relative_path_but_with_c_extension_osstring(&self) -> std::ffi::OsString {
        return self.relative_path_but_with_extension_osstring(".c");
    }
    fn relative_path_but_with_c_extension_string(&self) -> String {
        return self.relative_path_but_with_c_extension_osstring().into_string().unwrap();
    }

    fn relative_path_but_with_extension_osstring(&self, extension: &str) -> std::ffi::OsString {
        let mut res = self.path_relative_to_output_directory.file_stem().unwrap().to_owned();
        res.push(extension);
        return res;
    }
}

pub fn write_header_to_output_file_info(output_info: &mut Output_info_for_a_single_file) {
    output_info.content_string.push_str(
        format!(
            "// {} - Generated file by ITGBA \n",
            output_info.c_file_name_string()
        )
        .as_str(),
    );
}

pub fn write_tileset_to_output_file_info(tiledata_vec: &Vec<Tiledata>, output_info: &mut Output_info_for_a_single_file, use_hex_notation: bool) {

    output_info.content_string.push_str(
        format!("const unsigned char {}[] = \n", output_info.c_file_name_string().as_str()).as_str()
    );

    // Iterate over the remaining tiles with starting at index (1,0) to (image_width_in_tiles,0), (0,1) and so on...
    for (tile_index, tile_data) in tiledata_vec.iter().enumerate() {
        output_info.content_string.push_str(format!("\n\t// Tile {}\n", tile_index).as_str());

        for y in 0..8 {
            if (y  % 2 == 0) {
                output_info.content_string.push_str("\t");
            }
            // write 2 bytes corresponding to the line x
            let mut first_byte: u8 = 0; // stores the least significant bits of the palette indices
            let mut second_byte: u8 = 0; // stores the highest significant bits " 

            for x in 0..8 {
                let palette_index = tile_data.get(x,y);
                first_byte += ((palette_index & 1u8) << 7) >> x;
                second_byte += (((palette_index & 2u8) >> 1) << 7) >> x;
            }

            output_info.content_string
                .push_str(match use_hex_notation {
                    true => format!("{:#04x}, {:#04x}, ", first_byte, second_byte),
                    false => format!("{:#010b}, {:#010b}, ", first_byte, second_byte),
        }.as_str());

            if ((y + 1) % 2 == 0) {
                output_info.content_string.push_str(format!(" // Line {}-{}\n", y-1, y).as_str());
            }
        }
    }

}

