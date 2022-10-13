pub struct Vector3Usize {
    pub x: usize,
    pub y: usize,
    pub z: usize
}

impl Vector3Usize {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        return Self {
            x, y, z
        }
    }
}