mod vector3uint;
mod vector3float;
mod sphere;
//mod cpu;
mod gpu;

use std::{fs::File, io::{Write, Error, BufWriter}, time::Instant};

use byteorder::{WriteBytesExt, BigEndian};
use rand::seq::SliceRandom;
use sphere::Sphere;
use vector3uint::Vector3Usize;
use std::env;

pub struct GeneratorData {
    pub variation_count: u32,
    pub sphere_count: i32,
    pub min_sphere_radius: f32,
    pub max_sphere_radius: f32,
    pub empty_space: u32,
    pub noise_span: u32,
    pub sphere_group_count: i32,
    pub pixel_dimensions: Vector3Usize,
    pub generation_name: String
}

fn generate_spheres(gen_data: &GeneratorData) -> Vec<Sphere> {
    let sphere_count = gen_data.sphere_count;
    let min_rad = gen_data.min_sphere_radius;
    let max_rad = gen_data.max_sphere_radius;


    let mut spheres = Vec::new();

    let mut rand_rng = rand::thread_rng();
    let mut densities: Vec<u32> = (gen_data.empty_space+gen_data.noise_span..=255).collect();
    densities.shuffle(&mut rand_rng);

    let density_size = densities.len();
    for i in 0..sphere_count {
        let sphere = Sphere::generate_sphere(min_rad, max_rad, densities[i as usize % density_size], i);
        spheres.push(sphere);
    }
    spheres.sort();

    println!("Generated {} spheres", spheres.len());
    return spheres;
}

fn write_raw(filename: &str, texture: Vec<u32>) -> Result<(), Error> {
    println!("Writing density texture...");
    let time = Instant::now();

    let texture_u8: Vec<u8> = texture.into_iter().map(|x| x as u8).collect();
    
    match File::create(filename) {
        Ok(mut f) => {
            match f.write_all(&texture_u8) {
                Ok(_) => (),
                Err(e) => {
                    return Err(e);
                }
            }
        },
        Err(e) => {
            return Err(e);
        }
    };
    
    println!("Written density texture to file ({} secs)", time.elapsed().as_secs_f32());
    return Ok(());
}

fn write_segmentation(filename: &str, texture: Vec<i32>) -> Result<(), Error> {
    println!("Writing material texture...");
    let time = Instant::now();

    match File::create(filename) {
        Ok(f) => {
            let mut buf = BufWriter::new(f);
            for t in texture {
                match buf.write_i32::<BigEndian>(t) {
                    Ok(_) => (),
                    Err(e) => {
                        return Err(e);
                    }
                };
            }
        },
        Err(e) => {
            return Err(e);
        }
    };

    println!("Written material texture to file ({} secs)", time.elapsed().as_secs_f32());
    return Ok(());
}

fn generation(generator_data: &GeneratorData, i: u32) -> Result<(), Error> {
    println!("Generating {}. iteration...", i+1);
    let time = Instant::now();

    let gen_name = &generator_data.generation_name;
    let inst_count = generator_data.sphere_count;
    let dims = &generator_data.pixel_dimensions;

    let volume_filename = format!("{}_{}_i{}_{}x{}x{}.raw",
        gen_name, i, inst_count, dims.x, dims.y, dims.z);
    let material_filename = format!("{}_{}_i{}_{}x{}x{}.sgm",
        gen_name, i, inst_count, dims.x, dims.y, dims.z);

    let spheres = generate_spheres(&generator_data);
    let (material_tex, volume_tex) = gpu::generate_volume_textures_cl(&generator_data, &spheres);

    match write_raw(&volume_filename, volume_tex) {
        Ok(_) => (),
        Err(e) => {
            return Err(e);
        }
    };
    match write_segmentation(&material_filename, material_tex) {
        Ok(_) => (),
        Err(e) => {
            return Err(e);
        }
    };

    println!("Generation complete in {} secs", time.elapsed().as_secs_f32());
    return Ok(());
}

fn get_params_from_args() -> Option<GeneratorData> {
    let args: Vec<String> = env::args().collect();
    let mut gen_data = GeneratorData {
        variation_count: 1,
        sphere_count: 100,
        min_sphere_radius: 0.001,
        max_sphere_radius: 0.1,
        empty_space: 30,
        noise_span: 10,
        sphere_group_count: 1,
        pixel_dimensions: Vector3Usize::new(256, 256, 256),
        generation_name: String::from("untitled")
    };

    for i in 0..args.len() {
        if args[i] == String::from("-o") {
            if i+1 < args.len() {
                gen_data.generation_name = args[i+1].clone();
            } else {
                println!("Warning: Output name not specified. Defaulting to '{}'", gen_data.generation_name);
            }
        }

        else if args[i] == String::from("-v") {
            if i+1 < args.len() {
                gen_data.variation_count = match args[i+1].parse() {
                    Ok(c) => c,
                    Err(_) => {
                        println!("Warning: Number of variations not a valid integer, defaulting to {}", gen_data.variation_count);
                        gen_data.variation_count
                    }
                }
            } else {
                println!("Warning: Number of variations not specified, defaulting to {}", gen_data.variation_count);
            }
        }

        else if args[i] == String::from("-i") {
            if i+1 < args.len() {
                gen_data.sphere_count = match args[i+1].parse() {
                    Ok(c) => c,
                    Err(_) => {
                        println!("Warning: Number of instances not a valid integer, defaulting to {}", gen_data.sphere_count);
                        gen_data.sphere_count
                    }
                }
            } else {
                println!("Warning: Number of instances not specified, defaulting to {}", gen_data.sphere_count);
            }
        }

        else if args[i] == String::from("-r") {
            if i+2 < args.len() {
                gen_data.min_sphere_radius = match args[i+1].parse() {
                    Ok(c) => c,
                    Err(_) => {
                        println!("Warning: Bottom radius limit not a valid float, defaulting to {}", gen_data.min_sphere_radius);
                        gen_data.min_sphere_radius
                    }
                };
                gen_data.max_sphere_radius = match args[i+2].parse() {
                    Ok(c) => c,
                    Err(_) => {
                        println!("Warning: Top radius limit not a valid float, defaulting to {}", gen_data.max_sphere_radius);
                        gen_data.max_sphere_radius
                    }
                };
            }
            else if i+1 < args.len() {
                gen_data.max_sphere_radius = match args[i+1].parse() {
                    Ok(c) => {
                        gen_data.min_sphere_radius = 0.0;
                        c
                    },
                    Err(_) => {
                        println!("Warning: Top radius limit not a valid float, defaulting to {}", gen_data.max_sphere_radius);
                        gen_data.max_sphere_radius
                    }
                }
            }
            else {
                println!("Warning: Radius limits not specified, defaulting to {} and {}", gen_data.min_sphere_radius, gen_data.max_sphere_radius);
            }
        }

        else if args[i] == String::from("-n") {
            if i+1 < args.len() {
                gen_data.empty_space = match args[i+1].parse() {
                    Ok(c) => c,
                    Err(_) => {
                        println!("Warning: Max empty space density value not a valid integer, defaulting to {}", gen_data.empty_space);
                        gen_data.empty_space
                    }
                }
            } else {
                println!("Warning: Max empty space density value not specified, defaulting to {}", gen_data.empty_space);
            }
        }

        else if args[i] == String::from("-q") {
            if i+1 < args.len() {
                gen_data.noise_span = match args[i+1].parse() {
                    Ok(c) => c,
                    Err(_) => {
                        println!("Warning: Quality variability not a valid integer, defaulting to {}", gen_data.noise_span);
                        gen_data.noise_span
                    }
                }
            } else {
                println!("Warning: Quality variability not specified, defaulting to {}", gen_data.noise_span);
            }
        }

        else if args[i] == String::from("-d") {
            if i+1 < args.len() {
                let dims: Vec<&str> = args[i+1].split("x").collect();
                let mut bad_dim = String::new();

                if dims.len() == 3 {
                    let x: usize = match dims[0].parse() {
                        Ok(c) => c,
                        Err(_) => {
                            bad_dim.push_str("x");
                            gen_data.pixel_dimensions.x
                        }
                    };
                    let y: usize = match dims[1].parse() {
                        Ok(c) => c,
                        Err(_) => {
                            bad_dim.push_str("y");
                            gen_data.pixel_dimensions.y
                        }
                    };
                    let z: usize = match dims[2].parse() {
                        Ok(c) => c,
                        Err(_) => {
                            bad_dim.push_str("z");
                            gen_data.pixel_dimensions.z
                        }
                    };
                    gen_data.pixel_dimensions = Vector3Usize::new(x, y, z);
                    if !bad_dim.is_empty() {
                        println!("Warning: Some of the provided volume dimensions were not valid, these dimensions are {}", bad_dim);
                    }
                } else {
                    println!("Warning: Not all volume dimensions were provided, defaulting to {}x{}x{}", gen_data.pixel_dimensions.x, gen_data.pixel_dimensions.y, gen_data.pixel_dimensions.z);
                }
            } else {
                println!("Warning: Volume dimensions not provided, defaulting to {}x{}x{}", gen_data.pixel_dimensions.x, gen_data.pixel_dimensions.y, gen_data.pixel_dimensions.z);
            }
        }

        else if args[i] == "-h" {
            println!("-----------------------------------------------------------");
            println!("This is a small tool for the creation of testing volumes.\n");
            println!("This tool outputs two textures:");
            println!("  * .raw file with volumetric data as a sequence of unsigned 8-bit integers");
            println!("  * .sgm file with space segmented into classes as a sequence of signed 64-bit integers\n");
            println!("The supported parameters are:");
            println!("  * -h  Shows this help message.");
            println!("  * -o  Output name to append to generated files. Defaults to {}.", gen_data.generation_name);
            println!("  * -v  Number of variations. Generates and exports this many volumes with given settings, but different hidden parameters (instance positions etc.). Defaults to {}.", gen_data.variation_count);
            println!("  * -i  Number of instances (spheres) to put inside the volume. Defaults to {}.", gen_data.sphere_count);
            println!("  * -r  Interval from which to uniformly sample a single sphere radius. Specified as two floats, separated with a whitespace. If only one float is provided, the range is [0, given radius]. Defaults to [{}-{}]", gen_data.min_sphere_radius, gen_data.max_sphere_radius);
            println!("  * -n  Value in range [0,255], up to which the values are considered empty space. Can be treated as the largest possible density of the empty space. Defaults to {}.", gen_data.empty_space);
            println!("  * -q  Quality of the volume, given as the amount of density the assigned instance density can differ by. Larger means more noise. Defaults to {}.", gen_data.noise_span);
            println!("  * -d  Dimensions of the volume, provided as three integers separated by 'x'. Defaults to {}x{}x{}.", gen_data.pixel_dimensions.x, gen_data.pixel_dimensions.y, gen_data.pixel_dimensions.z);
            println!("-----------------------------------------------------------");
            return None;
        }
    }
    return Some(gen_data);
}

fn main() {
    let generator_data = match get_params_from_args() {
        Some(g) => g,
        None => {
            return;
        }
    };
    println!("Note: To view all available options, run this tool with parameter -h.\n");

    let variation_count = generator_data.variation_count;

    for i in 0..variation_count {
        match generation(&generator_data, i) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error: {}", e);
                return;
            }
        }
    }

    println!("Exiting...");
}
