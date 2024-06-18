use std::collections::{HashMap, LinkedList};
use std::path::Path;

use clap::builder::styling::Color;
use image::{DynamicImage, Rgb};
use derive_more::{Deref, DerefMut};

use crate::helper::*;
use crate::input_data_representation_types::*;

const ALL_DIGITS_STR: &str = "0123456789";
const CXX_VALID_IDENTIFIER_CHARACTERS: &str = "0123456789abcdefghijklmnopqrstuvwyz_ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const IGNORE_PREFIX: &str = "_ignore";


#[derive(Deref, DerefMut)]
pub struct TileSearchmap(HashMap<TileColorArray, TileSearchmapValue>);

pub fn read_tileset_info_from_path(path: &Path) -> (Vec<TileInfo>, ColorPalette) {
    let mut tile_info_vec = Vec::new();
    let mut color_palette: ColorPalette = unsafe { std::mem::uninitialized()};

    let mut read_palette_from_rgb_image = |image: image::RgbImage| {
        let mut pixels = image.pixels();

        color_palette[0] = *pixels.next().unwrap();
        color_palette[1] = *pixels.next().unwrap();
        color_palette[2] = *pixels.next().unwrap();
        color_palette[3] = *pixels.next().unwrap();

    };

    if path.is_dir() {
        let mut directory_entry_vec = std::fs::read_dir(path)
        .expect(format!("Couldn't read reference tileset directory members from path: {}", path.imm_to_str()).as_str())
        .map(|x| x.unwrap())
        .collect::<Vec<std::fs::DirEntry>>();

        directory_entry_vec.sort_by(|x, y| (x.file_name().to_str().unwrap()).cmp(
            y.file_name().to_str().unwrap()
        ));

        if directory_entry_vec.len() < 2 {
            panic!("Reference tileset directory should contain at least 2 members, a palette tile and a tileset tile");
        };

        let mut directory_entry_iterator = directory_entry_vec.into_iter();

        // first tile should contain the color palette
        let first_tile_image = rgbimage_from_path(&directory_entry_iterator.next().unwrap().path());

        read_palette_from_rgb_image(first_tile_image);

        for dir_entry in  directory_entry_iterator {
            let image = rgbimage_from_path(&dir_entry.path());

            if (!(image.width() == 8)) || (!(image.height() == 8)) {
                panic!("If the reference tileset is supplied via a directory path, all directory entries should be single tile 8x8 images. But\
                the image: \"{}\" is not 8x8", dir_entry.path().imm_to_str())
            }

            let file_name_stem: std::ffi::OsString = dir_entry.path().file_stem().unwrap().to_os_string();
            
            let file_name_stem_str = file_name_stem.to_str().unwrap();

            //Check if the file stem contains only valid characters
            let file_name_stem_is_valid_cxx_identifier = 
                (!file_name_stem_str.chars().any(|i| !CXX_VALID_IDENTIFIER_CHARACTERS.contains(i))) &&
                (!ALL_DIGITS_STR.contains(file_name_stem_str.chars().next().unwrap()));

            let file_name_has_ignore_prefix = if file_name_stem_str.len() >= IGNORE_PREFIX.len() {
                file_name_stem_str[0..file_name_stem_str.len()] == (*IGNORE_PREFIX)
            } else { false};

            if (!file_name_has_ignore_prefix) && (!file_name_stem_is_valid_cxx_identifier) {
                print_warning_once(format!("Some file names in the reference tileset directory are not valid C identifier names, and dont have an ignore prefix \"{}\" They will be ignored as well, meaning that theire tile index won't be accessible via a constant in the generated C file", IGNORE_PREFIX).as_str()) // TODO make this a one time warning
            }

            tile_info_vec.push(
                TileInfo {
                    color_array: read_tile_from_image(0, 0, &image, &color_palette),
                    name: if (!file_name_has_ignore_prefix) && file_name_stem_is_valid_cxx_identifier {
                        Some(file_name_stem_str.to_string())
                    } else {
                        None
                    }
                }
            )

        }

    } else {
        let reference_tileset_image: image::RgbImage = rgbimage_from_path(path);

        // Check that the image has valid dimensions
        if (reference_tileset_image.width() % 8 > 0 || reference_tileset_image.height() % 8 > 0) {
            panic!("Reference tileset's image dimensions aren't multiples of tile size (8)");
        }

        let image_width_in_tiles = reference_tileset_image.width() / 8;
        let image_height_in_tiles = reference_tileset_image.height() / 8;

        // Sanity check on tilemap size
        if image_width_in_tiles * image_height_in_tiles -1 > u8::MAX as u32 {
            panic!("Reference tileset is too big. The map should contain only 256 data tiles apart from the reference tile at maximum.")
        }

        if (image_width_in_tiles * image_height_in_tiles < 2) {
            panic!("Image needs to contain at least 2 tiles: A reference tile for mapping colors to the pallete indices 0-3 and at least one data tile")
        }




        // Iterate over the remaining tiles with starting at index (1,0),(2,0)... (image_width_in_tiles,0), (0,1) and so on...
        for tile_x in 0..image_width_in_tiles {
            for tile_y in 0..image_height_in_tiles {
                // skip tile (0,0)
                if (tile_x == 0 && tile_y == 0) {
                    continue;
                }


                tile_info_vec.push(
                    TileInfo {
                        color_array: read_tile_from_image(tile_x, tile_y, &reference_tileset_image, &color_palette),
                        name: None
                    }
                );
            }
        }

    }

    return (tile_info_vec, color_palette);
}

// Set up a hashmap that contains every version of a tile (original, x-flipped, y-flipped, x-flipped+y-flipped) and the corresponding tile index and flip information
pub fn tile_searchmap_from_tiledata_vec(tile_info_vec: Vec<TileInfo>) -> TileSearchmap {

    let mut tile_searchmap: TileSearchmap= TileSearchmap(HashMap::new());
    for (tile_index, tile_info) in tile_info_vec.into_iter().enumerate() {

        for x_flip in 0..2 {
            for y_flip in 0..2 {

                if(x_flip == 0) && (y_flip==0) {continue;}
                    
                let mut modified_tile_data: TileColorArray = unsafe {std::mem::uninitialized()};
                
                
                // Populate new TileColorArray
                for x in 0..8 { 
                    for y in 0..8 {
                        let new_x = x_flip * (7-x) + (1-x_flip) * x;
                        let new_y = y_flip * (7-y) + (1-y_flip) * y;

                        modified_tile_data.assign(new_x, new_y, tile_info.color_array.get(x,y));
                    }
                }

                tile_searchmap.insert(modified_tile_data, TileSearchmapValue {
                    x_flip: x_flip !=0,
                    y_flip: y_flip != 0,
                    tile_index: tile_index as u8,
                    link: None
                });

            }
        }

        // Insert original tile
        tile_searchmap.insert(tile_info.color_array, TileSearchmapValue {
            x_flip: false,
            y_flip: false,
            tile_index: tile_index as u8,
            link: None
        });

    }

    return tile_searchmap;
}

pub fn rgbimage_from_path(path: &Path) -> image::RgbImage {
    let image: DynamicImage = image::open(
            path
    ).expect(format!("Failed to open the path \"{}\"", path.imm_to_str()).as_str() );
    
    // GBC supports a 15-bit RGB (32768) colors (5-bits per channel)
    // Convert image to closest representation with 8-bits per channel
    let image: image::RgbImage = image.into_rgb8();
    return image;
}


pub fn read_tile_from_image(tile_index_x: u32, tile_index_y: u32, image: &image::RgbImage, color_palette: &ColorPalette) -> TileColorArray {
    let mut tile_color_array: TileColorArray;
    unsafe {
        tile_color_array = std::mem::uninitialized();
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
                panic!("Data tiles in image contain other colors than the palette (Error at tile: ({},{}) and relative pixel coordinates: ({},{}) i.e. ({},{}) in absolute pixel coordinates", tile_index_x, tile_index_y, x,y,cur_pixel_x, cur_pixel_y);
            };
            tile_color_array.assign(x,y,palette_index);
        }
    }

    return tile_color_array;
}

pub fn index_and_attribute_array_from_tilemap_image_path(tilemap_image_path: &Path, color_palette: &ColorPalette, tile_search_map: &HashMap<TileColorArray, TileSearchmapValue> ,allow_attributes_and_generate_attribute_array: bool) -> (TileIndexArray, Option<AttributeByteArray>) {

    let tilemap_image = rgbimage_from_path(tilemap_image_path);

    // sanity checks on image dimensions
    if((tilemap_image.width() % 8) > 0) || ((tilemap_image.height() % 8) > 0) {
        panic!(
            "Dimensions of tilemap from the path \"{}\" aren't multiples of 8 (tile size).", tilemap_image_path.imm_to_str()
        );
    }
    let tilemap_width = tilemap_image.width() / 8;
    let tilemap_height = tilemap_image.height() / 8;

    if (tilemap_width > 32) || (tilemap_height > 32) {
        panic!(
            "tilemap from the path \"{}\" is too big (max size: 32x32)", tilemap_image_path.imm_to_str()
        );
    }

    let mut tile_index_array: TileIndexArray = unsafe { std::mem::zeroed()};

    let mut attributes_byte_array: Option<AttributeByteArray> = match allow_attributes_and_generate_attribute_array {
        true => Some(unsafe {std::mem::uninitialized::<AttributeByteArray>()}),
        false => None
    };

    for x in 0..tilemap_width {
        for y in 0..tilemap_height {
            let current_tile_tiledata = read_tile_from_image(x,y,&tilemap_image, color_palette);
            match tile_search_map.get(&current_tile_tiledata) {
                Some(searchmap_value) => { // found matching tile in search map. Not all matches are allowed though depending on allow_attributes_and_generate_attribute_vector
                    let mut current_chain_link: &TileSearchmapValue = searchmap_value;

                    loop {
                        // Decide whether this tile index can and should be used
                        // This code will prefer unflipped tiles over flipped tiles
                        if (current_chain_link.is_unflipped() || (allow_attributes_and_generate_attribute_array && current_chain_link.link.is_none())) { // -> found matching tile
                            tile_index_array.assign(x,y, current_chain_link.tile_index);

                            if let Some(byte_array) = attributes_byte_array.as_mut() {

                                let mut attribute_byte: u8 = 0;
                                //	7	        6	    5	        4	    3	    210
                                //	Priority	Y flip	X flip		/       Bank	Color palette
                                attribute_byte += (current_chain_link.x_flip as u8) << 5;
                                attribute_byte += (current_chain_link.y_flip as u8) << 6;

                                byte_array.assign(x,y, attribute_byte);
                            }
                            break;
                        } else if let Some(link) =  searchmap_value.link.as_ref() {
                            current_chain_link = &link; 
                        } else {
                            panic!("tilemap from path \"{}\" contains a flipped tile from the reference tileset without an exact match at the tile index: ({},{}). Only the GBC allows for flipped tiles
                            via an additional attribute byte tilemap space in VRAM. Consider using the -gbc_map_with_attributes parameter instead to generate an 
                            attribute array in addition to the index array and allow for flipped tiles", tilemap_image_path.imm_to_str(),x,y);
                        }
                    }
                },
                None => {
                    panic!(
                        "tilemap from the path \"{}\" contains an unrecognized tile, that is not contained in the reference tileset, at tile index: ({},{})", tilemap_image_path.imm_to_str(), x,y 
                    );
                }
            }
        }
    } 

    return (tile_index_array, attributes_byte_array);
}




pub struct TileSearchmapValue {
    x_flip: bool,
    y_flip: bool,
    tile_index: u8,
    link: Option<Box<TileSearchmapValue>>
}

impl TileSearchmapValue {
    fn is_unflipped(&self) -> bool {
        (!self.x_flip) && (!self.y_flip)
    }
}