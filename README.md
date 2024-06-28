# ITGBA - ImageToGameboyAsset

A command line tool to help convert image data to a gameboy compatible format useful for gameboy games. At the moment ITGBA only generates .c files.

## Download


## Example usage

### Ex. 1: Tileset supplied as a single image + Tilemaps

    .                                                       #CWD layout
    ├── example_asset_directory                    
    │   ├── tileset.png          
    │   ├── tilemap_1.png         
    │   └── tilemap_2_gbc_with_attributes.png               # Tilemaps with attributes are only supported by the gbc. They might contain flipped tiles that aren't actually contained in the reference tileset.
    └── ...
```console
$ ITGBA --reference_tileset tileset.png --input_directory example_asset_directory --map tilemap_1.png --mwa tilemap_2_gbc_with_attributes.png --output_directory build
```
Results in:

    .
    ├── example_asset_directory                    
    │   └── ...
    ├── build
    │   ├── tileset.c
    │   ├── tilemap_1.c
    │   └── tilemap_2_gbc_map_with_attributes.c
    └── ...

### Ex. 2: Tileset supplied as a directory of 8x8 images (this allows for named tile indices!)
    .
    ├── example_asset_directory                    
    │   ├── tileset
    │   │   ├   tile_000 # First tile by lexicographic order needs to be the reference tile
    │   │   ├   __ignore_tile_001 # Since this tile is prefixed with __ignore, it's index will receive no named constant
    │   │   ├   valid_c_identifier.png # A constant for this tile's index is generated
    │   │   └── ...
    │   └── ...
    └── ...
```console
$ ITGBA --reference_tileset example_asset_directory/tileset
```
Results in:

    .
    ├── example_asset_directory                    
    │   └── ...
    ├── build
    │   ├── tileset.c
    │   └── ...
    └── ...

At the very minimum, a user needs to supply a tileset to ITGBA via the
required `--reference_tileset` option. Either a **single image** file should
be specified **or** a **folder that contains 8x8 images** for the individual
tiles. The tileset needs to include a "reference tile" that contains
the 4 palette colors in the upper left corner at the pixel coordinates
(0,0)-(3,0). 

```console
$ ITGBA --reference_tileset <path>
```
A tileset image could look like this: 

<img src="example_input/example_tileset_edited.png" alt="tileset.png" width="400" image-rendering= pixelated>

Tiles are ordered lexicographically by filename (by unicode code point) or
in case of specifying a single image from left-to-right, top-to-bottom.
**The reference tileset "tile" that encodes the palette indices of the colors
used in the actual tiles needs to be the first.**

Tilemaps can be supplied by the `--map` or `--mwa` (map with attributes) option.
Note that only the GBC supports attribute maps. Those may contain flipped tiles,
that are only indirectly contained in the reference tileset. **At the moment, multiple color
palettes are not supported and ITGBA does not translate the actual colors in the input
images, it only generates tile indices!!**.

One can change the CWD (current working directory) that ITGBA uses during reading 
by supplying the `--input_directory` flag.

Optionally one can supply an output directory via the `--output_directory` flag.
By default all generated files will just land in the (OUTPUT_DIRECTORY), but
an existing file hierarchy in the (INPUT_DIRECTORY) can be preserved via the
`--mimic_relative_paths_to_input_directory` flag.

For more usage info run `ITGBA --help`.

## Features



## Building

## Contribution

## License