#include "../include/honeycomb.h"
#include <stdint.h>

// BLOCK: (2, 2, 24)
// GRID:  (ceil(n_x/2), ceil(n_y/2), n_z)
extern "C" __global__ void
generate_hex_grid_betaf(DartIdType* out, size_t n_x, size_t n_y, size_t n_z, size_t n_out) {
  int offset_x           = 24;
  int offset_y           = offset_x * n_x;
  int offset_z           = offset_y * n_y;
  int const OFFSET_X     = 24;
  int const BETAS[24][4] = {
    // 1st face
    {3, 1, 4, 20},
    {-1, 1, 7, 22},
    {-1, 1, 10, 20},
    {-1, -3, 13, 18},
    // 2nd face
    {3, 1, -4, 8},
    {-1, 1, 14, 10},
    {-1, 1, 14, 8},
    {-1, -3, 2, 6},
    // 3rd face
    {3, 1, -7, 8},
    {-1, 1, -2, 10},
    {-1, 1, 13, 8},
    {-1, -3, 2, 6},
    // 4th face
    {3, 1, -10, -8},
    {-1, 1, -2, -6},
    {-1, 1, 8, -8},
    {-1, -3, 2, -10},
    // 5th face
    {3, 1, -13, -8},
    {-1, 1, -2, -6},
    {-1, 1, 3, -8},
    {-1, -3, -14, -10},
    // 6th face
    {3, 1, -14, -20},
    {-1, 1, -3, -18},
    {-1, 1, -8, -20},
    {-1, -3, -13, -22},
  };
  int offsets[24] = {-offset_y, -offset_y, -offset_y, -offset_y, -offset_z, -offset_z,
                     -offset_z, -offset_z, +offset_x, +offset_x, +offset_x, +offset_x,
                     +offset_z, +offset_z, +offset_z, +offset_z, -offset_x, -offset_x,
                     -offset_x, -offset_x, +offset_y, +offset_y, +offset_y, +offset_y};
  // cell coordinates in the generated grid
  uint64_t ix = threadIdx.x + blockIdx.x * blockDim.x;
  uint64_t iy = threadIdx.y + blockIdx.y * blockDim.y;
  uint64_t iz = blockIdx.z;
  // dart of the thread
  uint64_t dart = 1 + offset_x * ix + offset_y * iy + offset_z * iz + threadIdx.z;
  // boundary conditions
  int conds[24] = {
    iy == 0,       iy == 0,       iy == 0,       iy == 0,       iz == 0,       iz == 0,
    iz == 0,       iz == 0,       ix == n_x - 1, ix == n_x - 1, ix == n_x - 1, ix == n_x - 1,
    iz == n_z - 1, iz == n_z - 1, iz == n_z - 1, iz == n_z - 1, ix == 0,       ix == 0,
    ix == 0,       ix == 0,       iy == n_y - 1, iy == n_y - 1, iy == n_y - 1, iy == n_y - 1,
  };
  // beta images
  if (dart * 4 + 3 < n_out) {
    out[dart * 4]     = dart + BETAS[threadIdx.z][0];
    out[dart * 4 + 1] = dart + BETAS[threadIdx.z][1];
    out[dart * 4 + 2] = dart + BETAS[threadIdx.z][2];
    out[dart * 4 + 3] =
      conds[threadIdx.z] ? 0 : dart + BETAS[threadIdx.z][3] + offsets[threadIdx.z];
  }
}

// BLOCK: (2, 2, 24)
// GRID:  (ceil(n_x/2), ceil(n_y/2), n_z)
extern "C" __global__ void generate_hex_grid_vertices(
  CuVertex3* out, float lc_x, float lc_y, float lc_z, size_t n_x, size_t n_y, size_t n_out
) {
  int const OFFSETS[24][3] = {
    {0, 0, 0}, {1, 0, 0}, {1, 0, 1}, {0, 0, 1}, {1, 0, 0}, {0, 0, 0}, {0, 1, 0}, {1, 1, 0},
    {1, 0, 1}, {1, 0, 0}, {1, 1, 0}, {1, 1, 1}, {0, 0, 1}, {1, 0, 1}, {1, 1, 1}, {0, 1, 1},
    {0, 0, 0}, {0, 0, 1}, {0, 1, 1}, {0, 1, 0}, {1, 1, 0}, {0, 1, 0}, {0, 1, 1}, {1, 1, 1},
  };
  int offset_x = 24;
  int offset_y = offset_x * n_x;
  int offset_z = offset_y * n_y;

  // cell coordinates in the generated grid
  uint64_t ix = threadIdx.x + blockIdx.x * blockDim.x;
  uint64_t iy = threadIdx.y + blockIdx.y * blockDim.y;
  uint64_t iz = blockIdx.z;
  // dart of the thread
  uint64_t dart = 1 + offset_x * ix + offset_y * iy + offset_z * iz + threadIdx.z;
  // compute the vertex associated to every single dart;
  // we'll filter useful values when building on the host
  if (dart < n_out) {
    out[dart] = {
      lc_x * (ix + OFFSETS[threadIdx.z][0]),
      lc_y * (iy + OFFSETS[threadIdx.z][1]),
      lc_z * (iz + OFFSETS[threadIdx.z][2])
    };
  }
}
