#include "../include/honeycomb.h"
#include <stdint.h>

// BLOCK: (4, 4, 4)
// GRID:  (ceil(n_x/2), ceil(n_y/2), 1)
extern "C" __global__ void generate_2d_grid_betaf(DartIdType* out, size_t n_x, size_t n_y, size_t n_out) {
    const int BETAS[4][3] = { {3, 1, 2}, {-1, 1, 6}, {-1, 1, -2}, {-1, -3, -6} };
    const int NX[4]       = { -4, 0, 4, 0 };
    // cell coordinates in the generated grid
    uint64_t ix = threadIdx.x + blockIdx.x * blockDim.x;
    uint64_t iy = threadIdx.y + blockIdx.y * blockDim.y;
    // dart of the thread
    uint64_t dart = 1 + 4 * ix + 4 * n_x * iy + threadIdx.z;
    // boundary conditions
    int conds[4] = { iy == 0, ix == n_x - 1, iy == n_y - 1, ix == 0 };
    // beta images
    if (dart*3 + 2 < n_out) {
        out[dart*3]   = dart + BETAS[threadIdx.z][0];
        out[dart*3+1] = dart + BETAS[threadIdx.z][1];
        out[dart*3+2] = conds[threadIdx.z] ? 0 : dart + BETAS[threadIdx.z][2] + NX[threadIdx.z] * n_x;
    }
}

// BLOCK: (4, 4, 4)
// GRID:  (ceil(n_x/2), ceil(n_y/2), 1)
extern "C" __global__ void generate_2d_grid_vertices(
    CuVertex2* out,
    float lc_x,
    float lc_y,
    size_t n_x,
    size_t n_out
) {
    const int OFFSETS[4][2] = { {0, 1}, {0, 0}, {1, 0}, {1, 1} };
    // cell coordinates in the generated grid
    uint64_t ix = threadIdx.x + blockIdx.x * blockDim.x;
    uint64_t iy = threadIdx.y + blockIdx.y * blockDim.y;
    // dart of the thread
    uint64_t dart = 1 + 4 * ix + 4 * n_x * iy + threadIdx.z;
    // compute the vertex associated to every single dart;
    // we'll filter useful values when building on the host
    if (dart < n_out) {
        out[dart] = {
            lc_x * (ix + OFFSETS[threadIdx.z][0]),
            lc_y * (iy + OFFSETS[threadIdx.z][1])
        };
    }
}
