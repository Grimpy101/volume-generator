#[derive(Clone)]
pub struct Vector3Float {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Vector3Float {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        return Self {
            x, y, z
        }
    }

    /*pub fn distance(v1: &Vector3Float, v2: &Vector3Float) -> f32 {
        let a = v1.x - v2.x;
        let b = v1.y - v2.y;
        let c = v1.z - v2.z;
        return (a*a + b*b + c*c).sqrt();
    }*/
}