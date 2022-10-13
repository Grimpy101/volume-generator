use std::cmp::Ordering;

use rand::Rng;

use crate::vector3float::Vector3Float;

#[derive(Clone)]
pub struct Sphere {
    id: i32,
    radius: f32,
    origin: Vector3Float,
    density: u32,
    pub bounding_box: ((f32, f32), (f32, f32), (f32, f32))
}

impl Sphere {
    pub fn radius(&self) -> f32 {
        return self.radius;
    }

    pub fn id(&self) -> i32 {
        return self.id;
    }

    pub fn density(&self) -> u32 {
        return self.density;
    }

    pub fn origin(&self) -> &Vector3Float {
        return &self.origin;
    }

    pub fn generate_sphere(min_rad: f32, max_rad: f32, density: u32, id: i32) -> Self {
        let mut rand = rand::thread_rng();

        let o = Vector3Float::new(rand.gen(), rand.gen(), rand.gen());
        let r = rand.gen::<f32>() * (max_rad - min_rad) + min_rad;

        let x_range = (o.x - r, o.x + r);
        let y_range = (o.y - r, o.y + r);
        let z_range = (o.z - r, o.z + r);

        return Self {
            id,
            radius: r,
            origin: o,
            density,
            bounding_box: (x_range, y_range, z_range)
        }
    }

    /*pub fn is_point_in_sphere(&self, p: &Vector3Float) -> bool {
        //println!("{}", Vector3Float::distance(p, &self.origin));
        if Vector3Float::distance(p, &self.origin) <= self.radius {
            return true;
        }
        return false;
    }*/
}

impl Eq for Sphere {}

impl PartialEq for Sphere {
    fn eq(&self, other: &Self) -> bool {
        return self.id == other.id && self.radius == other.radius;
    }
}

impl PartialOrd for Sphere {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.radius == other.radius {
            return Some(self.id.cmp(&other.id));
        }
        return self.radius.partial_cmp(&other.radius);
    }
}

impl Ord for Sphere {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.radius == other.radius {
            return self.id.cmp(&other.id);
        }
        return self.radius.partial_cmp(&other.radius).unwrap();
    }
}