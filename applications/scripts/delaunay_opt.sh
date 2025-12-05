#!/bin/sh

SCRIPT_DIR=${HCWORKDIR}/applications
TARGET_DIR=${HCWORKDIR}/target/release

mkdir out

SPATIAL_SORT=--enable-spatial-sort

if [ -n "${DISABLE_SORT}" ]; then
  unset SPATIAL_SORT
fi

# Delaunay (sequential)
#
# - STM overhead/cost
# - algorithm perf

${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    32 \
    --init-points 1024 \
    ${SPATIAL_SORT} \
    >> out/delaunay_seq.out


# Delaunay (weak scaling, large starting triangulation)
#
# # of threads    | 1    | 3    | 9     | 18    | 36    | 54     | 72
# seq. insertions | 512  | 1536 | 4608  | 9216  | 18432 | 27648  | 36864
# par. insertions | 2048 | 6144 | 18432 | 36864 | 73728 | 110592 | 147456
#
# - scalability
# - algorithm perf

# 1 -> 72

RAYON_NUM_THREADS=1 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    2048 \
    --init-points 512 \
    ${SPATIAL_SORT} \
    >> out/delaunay_sc_1.out
RAYON_NUM_THREADS=3 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    6144 \
    --init-points 1536 \
    ${SPATIAL_SORT} \
    >> out/delaunay_sc_3.out
RAYON_NUM_THREADS=9 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    18432 \
    --init-points 4608 \
    ${SPATIAL_SORT} \
    >> out/delaunay_sc_9.out
RAYON_NUM_THREADS=18 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    36864 \
    --init-points 9216 \
    ${SPATIAL_SORT} \
    >> out/delaunay_sc_18.out
RAYON_NUM_THREADS=36 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    73728 \
    --init-points 18432 \
    ${SPATIAL_SORT} \
    >> out/delaunay_sc_36.out
RAYON_NUM_THREADS=54 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    110592 \
    --init-points 27648 \
    ${SPATIAL_SORT} \
    >> out/delaunay_sc_54.out
RAYON_NUM_THREADS=72 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    147456 \
    --init-points 36864 \
    ${SPATIAL_SORT} \
    >> out/delaunay_sc_72.out

# 1 -> 288

# TODO: adjust problem size, and use tet grid init
#
#
# TODO: complete
