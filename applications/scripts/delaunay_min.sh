#!/bin/sh

SCRIPT_DIR=${HCWORKDIR}/applications
TARGET_DIR=${HCWORKDIR}/target/release

mkdir out

SPATIAL_SORT=--enable-spatial-sort

if [ -n "${DISABLE_SORT}" ]; then
  unset SPATIAL_SORT
fi

# Delaunay (weak scaling, minimal starting triangulation)
#
# # of threads    | 1    | 3    | 9     | 18    | 36    | 54     | 72
# par. insertions | 2048 | 6144 | 18432 | 36864 | 73728 | 110592 | 147456
# 
# - conflict cost
# - algorithm perf

# 1 -> 72

RAYON_NUM_THREADS=1 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    2048 \
    ${SPATIAL_SORT} \
    >> out/delaunay_min_1.out
RAYON_NUM_THREADS=3 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    6144 \
    ${SPATIAL_SORT} \
    >> out/delaunay_min_3.out
RAYON_NUM_THREADS=9 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    18432 \
    ${SPATIAL_SORT} \
    >> out/delaunay_min_9.out
RAYON_NUM_THREADS=18 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    36864 \
    ${SPATIAL_SORT} \
    >> out/delaunay_min_18.out
RAYON_NUM_THREADS=36 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    73728 \
    ${SPATIAL_SORT} \
    >> out/delaunay_min_36.out
RAYON_NUM_THREADS=54 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    110592 \
    ${SPATIAL_SORT} \
    >> out/delaunay_min_54.out
RAYON_NUM_THREADS=72 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    147456 \
    ${SPATIAL_SORT} \
    >> out/delaunay_min_72.out
