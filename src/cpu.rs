use std::f32::consts::PI;

use crate::{GeneratorData, sphere::Sphere, vector3float::Vector3Float};

fn normal_dist(mu: f32, sig: f32) -> f32 {
    let u1 = rand::random::<f32>();
    let u2 = rand::random::<f32>();

    let r = (-2.0 * u1.ln()).sqrt();
    let cos = (2.0 * PI * u2).cos();
    return mu + sig.sqrt() * (r * cos);
}

fn get_density_and_material(p: Vector3Float, spheres: &Vec<Sphere>, gen_data: &GeneratorData) -> (u32, i32) {
    let mut current_rad = f32::MAX;
    let mut material = i32::MAX;
    let mut density = 0;
    for sphere in spheres {
        if sphere.is_point_in_sphere(&p) {
            if sphere.radius() <= current_rad && sphere.id() < material {
                current_rad = sphere.radius();
                material = sphere.id();
                density = sphere.density();
            }
        }
    }

    if material == i32::MAX {
        material = -1;
        let mu = gen_data.empty_space as f32 / 2.0;
        density = normal_dist(mu, mu) as u32;
    } else {
        let mu = density as f32;
        let phi = gen_data.noise_span as f32;
        density = normal_dist(mu, phi) as u32;
    }

    return (density, material);
}

fn get_point(gen_data: &GeneratorData, i: usize, j: usize, k: usize) -> Vector3Float {
    let size_x = gen_data.pixel_dimensions.x as f32;
    let size_y = gen_data.pixel_dimensions.y as f32;
    let size_z = gen_data.pixel_dimensions.z as f32;

    let x = (1.0 / size_x) * (i as f32 + 0.5);
    let y = (1.0 / size_y) * (j as f32 + 0.5);
    let z = (1.0 / size_z) * (k as f32 + 0.5);

    return Vector3Float::new(x, y, z);
}

pub fn generate_volume_textures(gen_data: &GeneratorData, spheres: &Vec<Sphere>) -> (Vec<i32>, Vec<u32>) {
    let size_x = gen_data.pixel_dimensions.x;
    let size_y = gen_data.pixel_dimensions.y;
    let size_z = gen_data.pixel_dimensions.z;
    
    let mut material_texture = Vec::with_capacity(size_x * size_y * size_z);
    let mut density_texture = Vec::with_capacity(size_x * size_y * size_z);

    for i in 0..size_x {
        for j in 0..size_y {
            for k in 0..size_z {
                let p = get_point(gen_data, i, j, k);
                let (density, material) = get_density_and_material(p, spheres, gen_data);
                //println!("{} {}", density, material);
                material_texture.push(material);
                density_texture.push(density);
            }
        }
    }
    println!("Generated material texture and density texture");
    return (material_texture, density_texture);
}