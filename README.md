# Volume generator script

This is a small script for generating volumetric data, written in Rust. Currently supports only placing a defined number of spheres of arbitrary
size into the volume box at random locations. Outputs volume file and segmentation file.

## Building

Requires Cargo. Just run ``cargo build --release`` to generate the binary and run that in the command line.

## How to use

All additional information, parameters and options can be found by running the script with ``-h`` parameter.
