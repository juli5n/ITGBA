#![allow(warnings)]
#![feature(path_file_prefix)]
use image::{DynamicImage, Rgb};
use std::path::{
    Path,
    PathBuf
};
use std::ops::Deref;
use std::vec::Vec;
use std::{collections::HashMap, env};
use clap::Parser;
use derive_more::{Deref, DerefMut};

#[derive(Parser)]
#[command(name = "ITGBA")]
#[command(version)]
#[command(about = "ImageToGameBoyAsset(ITGBA) takes a tileset (that contains a reference tile for palette order)\
and generates a corresponding .c file that specifies the tileset in the data format that the gameboy uses.")]
struct Cl_parser {
    #[arg(short = 'i', long = "input", value_name = "path_list", num_args=1..)]
    input_file_paths: Vec<PathBuf>,
    #[arg(short = 'r', long = "reference_tileset", value_name = "path", required = true)]
    reference_tileset_path: PathBuf,
    #[arg(long = "hex",action)]
    use_hex: bool,
    #[arg(short = 'o', long = "output_directory", value_name = "path")]
    output_directory: Option<PathBuf>,
}



fn main() {
    // Parse command line arguments
    let mut parse_result = Cl_parser::parse();


    
    let image: DynamicImage = image::open(
            parse_result.reference_tileset_path.clone()
    )
    .expect(format!("Can't open reference-tileset-path: {}", parse_result.reference_tileset_path.as_os_str().to_str().unwrap()).as_str());

    for path_buf in parse_result.input_file_paths.iter() {
        println!("{}, ", path_buf.as_os_str().to_str().unwrap());
    }

    // GBC supports a 15-bit RGB (32768) colors (5-bits per channel)
    // Convert image to closest representation with 8-bits per channel
    let image: image::RgbImage = image.into_rgb8();

    // Check that the image has valid dimensions
    if (image.width() % 8 > 0 || image.height() % 8 > 0) {
        panic!("Image dimensions aren't multiples of tile size (8)");
    }

    let image_width_in_tiles = image.width() / 8;
    let image_height_in_tiles = image.height() / 8;

    if (image_width_in_tiles * image_height_in_tiles < 2) {
        panic!("Image needs to contain at least 2 tiles: A reference tile for mapping colors to the pallete indices 0-3 and at least one data tile")
    }

    let mut pixels = image.pixels();
    let mut color_palette: ColorPalette;

    unsafe {
        color_palette = std::mem::uninitialized();
    }

    color_palette[0] = *pixels.next().unwrap();
    color_palette[1] = *pixels.next().unwrap();
    color_palette[2] = *pixels.next().unwrap();
    color_palette[3] = *pixels.next().unwrap();

    let mut tile_data_vec: Vec<Tiledata> = Vec::new();

    // Iterate over the remaining tiles with starting at index (1,0) to (image_width_in_tiles,0), (0,1) and so on...
    for tile_x in 0..image_width_in_tiles {
        for tile_y in 0..image_height_in_tiles {
            // skip tile (0,0)
            if (tile_x == 0 && tile_y == 0) {
                continue;
            }


            tile_data_vec.push(
                read_tile_from_image(tile_x, tile_y, &image, &color_palette)
            );
        }
    }

    let mut reference_tileset_path_prefix: String  = String::from(parse_result.reference_tileset_path.file_prefix().unwrap().to_str().unwrap());
    let mut resulting_file_contents = String::new();
    resulting_file_contents.push_str(
        format!(
            "// Generated file by GBDK_ITGBA \n\
            const unsigned char {}[] = {{ \n",
            reference_tileset_path_prefix
        )
        .as_str(),
    );

    // Iterate over the remaining tiles with starting at index (1,0) to (image_width_in_tiles,0), (0,1) and so on...
    for (tile_index, tile_data) in tile_data_vec.iter().enumerate() {
        resulting_file_contents.push_str(format!("\n\t// Tile {}\n", tile_index).as_str());

        for x in 0..8 {
            if (x  % 2 == 0) {
                resulting_file_contents.push_str("\t");
            }
            // write 2 bytes corresponding to the line x
            let mut first_byte: u8 = 0; // stores the least significant bits of the palette indices
            let mut second_byte: u8 = 0; // stores the highest significant bits " 

            for i in 0..8 {
                let palette_index = tile_data.0[x * 8 + i];
                first_byte += ((palette_index & 1u8) << 7) >> i;
                second_byte += (((palette_index & 2u8) >> 1) << 7) >> i;
            }

            resulting_file_contents
                .push_str(match parse_result.use_hex{
                    true => format!("{:#04x}, {:#04x}, ", first_byte, second_byte),
                    false => format!("{:#010b}, {:#010b}, ", first_byte, second_byte),
        }.as_str());

            if ((x + 1) % 2 == 0) {
                resulting_file_contents.push_str(format!(" // Line {}-{}\n", x-1, x).as_str());
            }
        }
    }
    resulting_file_contents.push_str("\n}");

    let mut output_file_name = reference_tileset_path_prefix;
    output_file_name.push_str(".c");

    let mut output_file_path = PathBuf::from(output_file_name);

    // handle output_directory if specified
    if let Some(path) = parse_result.output_directory {
        // create directory and parent directories if they don't exist
        std::fs::create_dir_all(path.clone()).unwrap();
        output_file_path = path.join(output_file_path);
    }
    
    std::fs::write(output_file_path, resulting_file_contents);

    // Set up a hashmap that contains every version of a tile (original, x-flipped, y-flipped, x-flipped+y-flipped)
    let mut all_versions_of_reference_tiles: HashMap<Tiledata, FlipAttributes> = HashMap::new();
    for tile_data in tile_data_vec {

        for x_flip in 0..2 {
            for y_flip in 0..2 {

                if(x_flip == 0) && (y_flip==0) {continue;}
                    
                let mut modified_tile_data: Tiledata;
                unsafe { modified_tile_data = std::mem::uninitialized();} 
                
                // Populate new tiledata
                for x in 0..8 { 
                    for y in 0..8 {
                        let new_x = x_flip * (7-x) + (1-x_flip) * x;
                        let new_y = y_flip * (7-y) + (1-y_flip) * y;

                        modified_tile_data.assign(new_x, new_y, tile_data.get(x,y));
                    }
                }

                all_versions_of_reference_tiles.insert(modified_tile_data, FlipAttributes {
                    x_flip: x_flip !=0,
                    y_flip: y_flip != 0
                });

            }
        }

        // Insert original tile
        all_versions_of_reference_tiles.insert(tile_data, FlipAttributes {
            x_flip: false,
            y_flip: false,
        });

    }



    //	7	        6	    5	        4	    3	    210
    //	Priority	Y flip	X flip		/       Bank	Color palette

}


fn read_tile_from_image(tile_index_x: u32, tile_index_y: u32, image: &image::RgbImage, color_palette: &ColorPalette) -> Tiledata {
    let mut tile_data: Tiledata;
    unsafe {
        tile_data = std::mem::uninitialized();
    }

    // Iterate over every pixel of the tile
    for x in 0..8 {
        for y in 0..8 {
            let palette_index: u8 =  'l: {
                // Determine palette index
                let cur_pixel_x: u32 = tile_index_x * 8 + x as u32;
                let cur_pixel_y: u32 = tile_index_y * 8 + y as u32;
                let cur_pixel = *image.get_pixel(cur_pixel_x, cur_pixel_y);
                for i in 0..4 {
                    if (color_palette[i] == cur_pixel) {
                        break 'l i as u8; //return palette index
                    }
                }
                panic!("Data tiles in image contain other colors than the palette (Error at pixel coordinates: ({},{})", cur_pixel_x, cur_pixel_y);
            };
            tile_data.assign(x,y,palette_index);
        }
    }

    return tile_data;
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Tiledata([u8; 64]);


#[derive(Deref, DerefMut)]
struct ColorPalette([Rgb<u8>; 4]);

impl Tiledata {
    fn assign(&mut self, x: u8,y: u8, palette_index: u8) {
        self.0[(y*8 + x) as usize] = palette_index;
    }
    fn get(&self, x: u8,y: u8) -> u8 {
        self.0[(y*8 + x) as usize]
    }
}

struct FlipAttributes {
    x_flip: bool,
    y_flip: bool,
}
