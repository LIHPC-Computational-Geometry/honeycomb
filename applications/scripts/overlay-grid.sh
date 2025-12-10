#!/bin/sh

SCRIPT_DIR=${HCWORKDIR}/applications
TARGET_DIR=${HCWORKDIR}/target/release

mkdir out

# Overlay Grid (sequential)

RAYON_NUM_THREADS=1 ${TARGET_DIR}/overlay-grid \
    -n 1000 \
    -d 10 \
    >> out/overlay_grid_seq.out

# Overlay Grid (strong scaling)

# 1 -> 72

RAYON_NUM_THREADS=1 ${TARGET_DIR}/overlay-grid \
    -n 10000 \
    -d 10 \
    >> out/overlay_grid_sc_1.out
RAYON_NUM_THREADS=3 ${TARGET_DIR}/overlay-grid \
    -n 10000 \
    -d 10 \
    >> out/overlay_grid_sc_3.out
RAYON_NUM_THREADS=9 ${TARGET_DIR}/overlay-grid \
    -n 10000 \
    -d 10 \
    >> out/overlay_grid_sc_9.out
RAYON_NUM_THREADS=18 ${TARGET_DIR}/overlay-grid \
    -n 10000 \
    -d 10 \
    >> out/overlay_grid_sc_18.out
RAYON_NUM_THREADS=36 ${TARGET_DIR}/overlay-grid \
    -n 10000 \
    -d 10 \
    >> out/overlay_grid_sc_36.out
RAYON_NUM_THREADS=54 ${TARGET_DIR}/overlay-grid \
    -n 10000 \
    -d 10 \
    >> out/overlay_grid_sc_54.out
RAYON_NUM_THREADS=72 ${TARGET_DIR}/overlay-grid \
    -n 10000 \
    -d 10 \
    >> out/overlay_grid_sc_72.out
