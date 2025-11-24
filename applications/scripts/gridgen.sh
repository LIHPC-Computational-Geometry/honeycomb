#!/bin/sh

SCRIPT_DIR=${HCWORKDIR}/applications
TARGET_DIR=${HCWORKDIR}/target/release

mkdir out

# Grid generation
#
# - initialization time
# - memory usage
# - GPU speedup

RAYON_NUM_THREADS=72 ${TARGET_DIR}/generate-grid \
    --simple-precision \
    3d \
    256 256 256 \
    256 256 256 \
    >> out/grid_3d_cpu.out
RAYON_NUM_THREADS=72 ${TARGET_DIR}/generate-grid \
    --gpu --simple-precision \
    3d \
    256 256 256 \
    256 256 256 \
    >> out/grid_3d_gpu.out

