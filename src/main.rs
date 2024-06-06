#![allow(warnings)]
use std::{collections::HashMap, env};
use std::path::Path;
use std::vec::Vec;
use image::{
    DynamicImage,
    Rgb
};
fn main() {
    let mut filename_without_extension: String = String::from("Titlescreen"); // TODO
    // maybe use clap instead
    /*
    std::ffi::OsString first_argument = if(env::args_os().count() == 2){
        env::args_os().nth(1).unwrap();
    } else {
        panic!("Missing argument");
    };
    */
    println!("Working directory: {}", std::env::current_dir().unwrap().as_os_str().to_str().unwrap() );
    let image: DynamicImage= image::open(
        std::env::current_dir().unwrap().join(Path::new("Titlescreen.png"))
    ).unwrap();

    // GBC supports a 15-bit RGB (32768) colors (5-bits per channel)
    // Convert image to closest representation with 8-bits per channel
    let image: image::RgbImage= image.into_rgb8();

    // Check that the image has valid dimensions
    if(image.width() % 8 > 0 || image.height() % 8 > 0) {panic!("Image dimensions aren't multiples of tile size (8)");}

    let image_width_in_tiles = image.width()/8;
    let image_height_in_tiles = image.height()/8;

    if (image_width_in_tiles*image_height_in_tiles < 2) {panic!("Image needs to contain at least 2 tiles: A reference tile for mapping colors to the pallete indices 0-3 and at least one data tile")}
    
    let mut pixels = image.pixels();
    let mut color_palette: [Rgb<u8>; 4];

    unsafe {
        color_palette = std::mem::uninitialized();
    }

    color_palette[0] = *pixels.next().unwrap();
    color_palette[1] = *pixels.next().unwrap();
    color_palette[2] = *pixels.next().unwrap();
    color_palette[3] = *pixels.next().unwrap();

    let tile_data_vec: Vec<Tiledata> = Vec::new();


    // Iterate over the remaining tiles with starting at index (1,0) to (image_width_in_tiles,0), (0,1) and so on...
    for tile_x in 0..image_width_in_tiles { 
        for tile_y in 0..image_height_in_tiles {

            // skip tile (0,0)
            if(tile_x == 0 && tile_y == 0) {continue;}

            let mut tile_data: Tiledata;
            unsafe {
                tile_data = std::mem::uninitialized();
            }

            // Iterate over every pixel of the tile
            for x in 0..8 {
                for y in 0..8 {

                    tile_data.0[y*8 + x] = 'l:{
                        // Determine palette index
                        let cur_pixel_x: u32 = tile_x*8 + x as u32;
                        let cur_pixel_y: u32 = tile_y*8 + y as u32;
                        let cur_pixel = *image.get_pixel(cur_pixel_x, cur_pixel_y);
                        for i in 0..4 {
                            if (color_palette[i] == cur_pixel) {
                                break 'l i as u8; //return palette index
                            }
                        }
                        panic!("Data tiles in image contain other colors than the palette (Error at pixel coordinates: ({},{})", cur_pixel_x, cur_pixel_y);
                        
                    };
                }
            }


        }
    }

    let mut resulting_file_contents = String::new();
    resulting_file_contents.push_str(format!("// Generated file by GBDK_ITGBA \n
    const unsigned char {}[] = {{ \n", filename_without_extension).as_str());

    // Iterate over the remaining tiles with starting at index (1,0) to (image_width_in_tiles,0), (0,1) and so on...
    for tile_index in 0..tile_data_vec.len() { 

    }
    
    

    
    
}

struct Tiledata([u8; 64]);
