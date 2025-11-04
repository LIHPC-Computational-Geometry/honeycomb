#!/bin/sh

# This script is used to execute a benchmark suite that serve as a performace baseline for future optimizations
#
# It is composed of the following benchmarks:
# - grid generation
# - remesh
#   - sequential execution
#   - strong scaling
# - Delaunay triangulation
#   - sequential execution
#   - weak scaling (from minimal triangulation)
#   - weak scaling (from growing triangulation)
#
# Thread numbers are set for 4*GH chips

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
TARGET_DIR=${SCRIPT_DIR}/../target/release

mkdir out

# Grid generation
#
# - initialization time
# - memory usage
# - GPU speedup

RAYON_NUM_THREADS=72 ${TARGET_DIR}/generate-grid 3d \
    --simple-precision \
    256 256 256 \
    256 256 256 \
    >> out/grid_3d_cpu.out
RAYON_NUM_THREADS=72 ${TARGET_DIR}/generate-grid 3d \
    --gpu --simple-precision \
    256 256 256 \
    256 256 256 \
    >> out/grid_3d_gpu.out

# Remesh (sequential)
#
# - STM overhead/cost
#
# TODO: adjust target size & n-rounds

RAYON_NUM_THREADS=1 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_seq.out

# Remesh (strong scaling)
#
# - scalability of different operations
# - STM implementation effects
#
# TODO: adjust target size & n-rounds

# first set of runs: 1 to 72 threads (1 chip)

RAYON_NUM_THREADS=1 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_1.out
RAYON_NUM_THREADS=3 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_3.out
RAYON_NUM_THREADS=9 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_9.out
RAYON_NUM_THREADS=18 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_18.out
RAYON_NUM_THREADS=36 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_36.out
RAYON_NUM_THREADS=54 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_54.out
RAYON_NUM_THREADS=72 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_72.out

# second set of runs: 1 to 288 threads (1 node)

RAYON_NUM_THREADS=1 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_1.out
RAYON_NUM_THREADS=36 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_36.out
RAYON_NUM_THREADS=72 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_72.out
RAYON_NUM_THREADS=108 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_108.out
RAYON_NUM_THREADS=144 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_144.out
RAYON_NUM_THREADS=180 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_180.out
RAYON_NUM_THREADS=216 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_216.out
RAYON_NUM_THREADS=252 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_252.out
RAYON_NUM_THREADS=288 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_288.out

# Delaunay (sequential)
#
# - STM overhead/cost
# - algorithm perf

${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    32 \
    --init-points 1024 \
    >> out/delaunay_seq.out

# Delaunay (weak scaling, minimal triangulation)
#
# - conflict cost
# - algorithm perf

# 1 -> 72

RAYON_NUM_THREADS=1 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    2048 \
    >> out/delaunay_min_1.out
RAYON_NUM_THREADS=3 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    6144 \
    >> out/delaunay_min_3.out
RAYON_NUM_THREADS=9 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    18432 \
    >> out/delaunay_min_9.out
RAYON_NUM_THREADS=18 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    36864 \
    >> out/delaunay_min_18.out
RAYON_NUM_THREADS=36 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    73728 \
    >> out/delaunay_min_36.out
RAYON_NUM_THREADS=54 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    110592 \
    >> out/delaunay_min_54.out
RAYON_NUM_THREADS=72 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    147456 \
    >> out/delaunay_min_72.out

# Delaunay (weak scaling, large triangulation)
#
# - scalability
# - algorithm perf

# 1 -> 72

RAYON_NUM_THREADS=1 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    2048 \
    --init-points 512 \
    >> out/delaunay_singlechip_1.out
RAYON_NUM_THREADS=3 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    6144 \
    --init-points 1536 \
    >> out/delaunay_singlechip_3.out
RAYON_NUM_THREADS=9 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    18432 \
    --init-points 4608 \
    >> out/delaunay_singlechip_9.out
RAYON_NUM_THREADS=18 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    36864 \
    --init-points 9216 \
    >> out/delaunay_singlechip_18.out
RAYON_NUM_THREADS=36 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    73728 \
    --init-points 18432 \
    >> out/delaunay_singlechip_36.out
RAYON_NUM_THREADS=54 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    110592 \
    --init-points 27648 \
    >> out/delaunay_singlechip_54.out
RAYON_NUM_THREADS=72 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    147456 \
    --init-points 36864 \
    >> out/delaunay_singlechip_72.out

# 1 -> 288

# TODO: adjust problem size, and use tet grid init

zip -r out.zip out/*
