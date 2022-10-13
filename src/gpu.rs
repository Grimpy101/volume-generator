use std::{ptr};

use opencl3::{device::{CL_DEVICE_TYPE_GPU, Device}, context::{Context}, command_queue::{CommandQueue, CL_QUEUE_PROFILING_ENABLE}, program::Program, kernel::{Kernel, ExecuteKernel}, types::{cl_float, CL_BLOCKING, cl_event, cl_uint, cl_int, CL_NON_BLOCKING}, memory::{Buffer, CL_MEM_READ_ONLY, CL_MEM_WRITE_ONLY}, platform::get_platforms};

use crate::{GeneratorData, sphere::{Sphere}};

fn init() -> Result<(Context, CommandQueue, Kernel), String> {
    let platforms = match get_platforms() {
        Ok(p) => p,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    let platform = match platforms.first() {
        Some(p) => p,
        None => {
            return Err("No platforms found".to_string());
        }
    };

    let devices = match platform.get_devices(CL_DEVICE_TYPE_GPU) {
        Ok(d) => d,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    let device = match (*devices).first() {
        Some(d) => {
            Device::new(*d)
        },
        None => {
            return Err("Missing devices".to_string());
        }
    };

    let context = match Context::from_device(&device) {
        Ok(c) => c,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    let queue = match CommandQueue::create_default_with_properties(
        &context, CL_QUEUE_PROFILING_ENABLE, 0
    ) {
        Ok(q) => q,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    let kernel_contents = include_str!("kernel.cl");

    let program = match Program::create_and_build_from_source(&context, &kernel_contents, "") {
        Ok(p) => p,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    let kernel = match Kernel::create(&program, "main") {
        Ok(k) => k,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    return Ok((context, queue, kernel));
}

pub fn generate_volume_textures_cl(gen_data: &GeneratorData, spheres: &Vec<Sphere>) -> (Vec<i32>, Vec<u32>) {
    let size_x = gen_data.pixel_dimensions.x;
    let size_y = gen_data.pixel_dimensions.y;
    let size_z = gen_data.pixel_dimensions.z;
    let size = size_x * size_y * size_z;

    let dims = [
        gen_data.pixel_dimensions.x as u32,
        gen_data.pixel_dimensions.y as u32,
        gen_data.pixel_dimensions.z as u32
    ];

    let (context, queue, kernel) = match init() {
        Ok(c) => c,
        Err(e) => {
            panic!("{}", e);
        }
    };

    let mut spheres_vec = Vec::with_capacity(spheres.len() * 6);

    for sphere in spheres {
        spheres_vec.push(sphere.origin().x);
        spheres_vec.push(sphere.origin().y);
        spheres_vec.push(sphere.origin().z);
        spheres_vec.push(sphere.radius());
        spheres_vec.push(sphere.id() as f32);
        spheres_vec.push(sphere.density() as f32);
    }

    /*let mut rand_vec: Vec<f32> = Vec::with_capacity(size);
    for _ in 0..size {
        rand_vec.push(rand::random());
    }*/

    let empty_space: cl_uint = gen_data.empty_space;
    let quality: cl_uint = gen_data.noise_span;
    let sphere_count: cl_int = spheres_vec.len() as i32;

    let mut dim_buffer = unsafe {
        match Buffer::<cl_uint>::create(&context, CL_MEM_READ_ONLY, 3, ptr::null_mut()) {
            Ok(b) => b,
            Err(e) => {
                panic!("{}", e);
            }
        }
    };
    let mut sph_buffer = unsafe {
        match Buffer::<cl_float>::create(&context, CL_MEM_READ_ONLY, spheres_vec.len(), ptr::null_mut()) {
            Ok(b) => b,
            Err(e) => {
                panic!("{}", e);
            }
        }
    };
    
    let den_buffer = unsafe {
        match Buffer::<cl_uint>::create(&context, CL_MEM_WRITE_ONLY, size, ptr::null_mut()) {
            Ok(b) => b,
            Err(e) => {
                panic!("{}", e);
            }
        }
    };
    let mat_buffer = unsafe {
        match Buffer::<cl_int>::create(&context, CL_MEM_WRITE_ONLY, size, ptr::null_mut()) {
            Ok(b) => b,
            Err(e) => {
                panic!("{}", e);
            }
        }
    };

    unsafe {
        match queue.enqueue_write_buffer(&mut dim_buffer, CL_BLOCKING, 0, &dims, &[]) {
            Ok(_) => (),
            Err(e) => {
                panic!("{}", e);
            }
        };
    }

    let wait_event = unsafe {
        match queue.enqueue_write_buffer(&mut sph_buffer, CL_BLOCKING, 0, &spheres_vec, &[]) {
            Ok(q) => q,
            Err(e) => {
                panic!("{}", e);
            }
        }
    };

    let kernel_event = unsafe {
        match ExecuteKernel::new(&kernel)
            .set_arg(&empty_space)
            .set_arg(&quality)
            .set_arg(&sphere_count)
            .set_arg(&dim_buffer)
            .set_arg(&sph_buffer)
            .set_arg(&den_buffer)
            .set_arg(&mat_buffer)
            .set_global_work_size(size)
            .set_wait_event(&wait_event)
            .enqueue_nd_range(&queue) {
                Ok(k) => k,
                Err(e) => {
                    panic!("{}", e);
                }
            }
    };

    let mut events: Vec<cl_event> = Vec::default();
    events.push(kernel_event.get());

    let mut den_result: Vec<cl_uint> = vec![0; size];
    let mut mat_result: Vec<cl_int> = vec![0; size];

    let read_event_1 = unsafe {
        match queue.enqueue_read_buffer(&den_buffer, CL_NON_BLOCKING, 0, &mut den_result, &events) {
            Ok(r) => r,
            Err(e) => {
                panic!("{}", e);
            }
        }
    };
    let read_event_2 = unsafe {
        match queue.enqueue_read_buffer(&mat_buffer, CL_NON_BLOCKING, 0, &mut mat_result, &events) {
            Ok(r) => r,
            Err(e) => {
                panic!("{}", e);
            }
        }
    };

    read_event_1.wait().unwrap();
    read_event_2.wait().unwrap();

    println!("Generated material texture and density texture");
    return (mat_result, den_result);
}