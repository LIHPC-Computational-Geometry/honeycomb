#!/bin/sh

# This script is used to execute a benchmark suite that serve as a performance baseline for future optimizations
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
# - Overlay grid mesh
#   - sequential execution
#   - strong scaling
#
# Thread numbers are set for 4*GH chips. `HCWORKDIR` is expected to be set to the repo's path.

SCRIPT_DIR=${HCWORKDIR}/applications/scripts
TARGET_DIR=${HCWORKDIR}/target/release

mkdir out

# Grid generation

sh ${SCRIPT_DIR}/gridgen.sh

# Remesh pipeline

sh ${SCRIPT_DIR}/remesh.sh

# Delaunay

sh ${SCRIPT_DIR}/delaunay.sh

# Overlay grid mesh

sh ${SCRIPT_DIR}/overlay-grid.sh


zip -r out.zip out/*
