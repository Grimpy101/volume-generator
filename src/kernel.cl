typedef struct point {
    float x;
    float y;
    float z;
} Point;

float uniform_rand(int* seed) {
    const int a = 16807;
    const int m = 2147483647;

    *seed = ((long) *seed * (long) a) % m;
    
    return (float) *seed / (float) m;
}

float normal_rand(float mu, float sig, int* seed) {
    float u1 = uniform_rand(seed);
    float u2 = uniform_rand(seed);

    float r = sqrt(-2.0 * native_log(u1));
    float cos = cospi(2.0 * u2);
    return mu + sqrt(sig) * (r * cos);
}

bool is_point_in_sphere(float3 o, float3 p, float r) {
    return distance(o, p) <= r;
}

kernel void main(
    uint empty_space,
    uint quality,
    int sphere_count,
    global int* dims,
    global float* spheres,
    global uint* densities,
    global int* materials
    ) {
        const size_t index = get_global_id(0);

        int rand_seed = index;

        float i = (float)(index % dims[0]);
        float j = (float)((index / dims[0]) % dims[1]);
        float k = (float)(index / (dims[0] * dims[1]));

        float3 p = {
            (1.0 / dims[0]) * (i + 0.5),
            (1.0 / dims[1]) * (j + 0.5),
            (1.0 / dims[2]) * (k + 0.5)
        };

        int current_id = 0;
        uint current_density = 0;

        for (uint in = 0; in < sphere_count; in += 6) {
            float3 o = {
                spheres[in],
                spheres[in+1],
                spheres[in+2]
            };
            float radius = spheres[in+3];
            int id = (int) spheres[in+4];
            uint density = (uint) spheres[in+5];

            if (is_point_in_sphere(o, p, radius)) {
                current_id = id;
                current_density = density;
                break;
            }
        }

        if (current_id == 0) {
            float mu = (float) empty_space / 2.0;
            current_density = (uint) normal_rand(mu, mu, &rand_seed);
        } else {
            float mu = (float) current_density;
            float phi = (float) quality;
            current_density = (uint) normal_rand(mu, phi, &rand_seed);
        }

        densities[index] = current_density;
        materials[index] = current_id;
}