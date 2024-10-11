import subprocess
import sys

# Definitions

GRISUBAL = "./target/release/grisubal"
GRISUBAL_PROF = "./target/profiling/grisubal"
FILE_PATH = "./examples/shape.vtk"

CSV_HEADER = "ImportVTK, BuildGeometry, DetectOrientation, ComputeOverlappingGrid, RemoveRedundantPoi, BuildMeshTot, BuildMeshInit, BuildMeshIntersecData, BuildMeshInsertIntersec, BuildMeshEdgeData, BuildMeshInsertEdge, Clip, Cleanup"

FIXED_SIZE = "1."
SIZE_RANGE = [x / 10 for x in range(1, 11)]

PERF_FREQ = "997"


# Benchmarks suite

def fixed():
    # avg runtime using criterion
    subprocess.run(["mkdir", "fixed"])
    subprocess.run(["cargo", "bench", "--bench", "grisubal"], stdout=open("fixed/criterion.txt", "w"))
    # avg runtime per section (50 samples)
    subprocess.run(["cargo", "build", "--release", "--features=profiling", "--bin=grisubal"])
    with open("fixed/sections.csv", "w") as f:
        subprocess.run(["echo", CSV_HEADER], stdout=f)
    for _ in range(50):
        with open("fixed/sections.csv", "a") as f:
            subprocess.run([GRISUBAL, FILE_PATH, FIXED_SIZE, FIXED_SIZE], stdout=f)
    # perf + flamegraph
    subprocess.run(
        ["perf", "record", "-o", "fixed/perf.data", "-F", PERF_FREQ, "--call-graph", "dwarf", GRISUBAL_PROF, FILE_PATH,
         FIXED_SIZE, FIXED_SIZE])
    subprocess.run(["flamegraph", "--flamechart", "--perfdata", "fixed/perf.data", "-o", "fixed/fg.svg"])
    # heaptrack
    subprocess.run(["heaptrack", "-o", "fixed/ht", GRISUBAL_PROF, FILE_PATH, FIXED_SIZE, FIXED_SIZE])
    return


def grid():
    # avg runtimes using criterion
    subprocess.run(["mkdir", "grid"])
    subprocess.run(["cargo", "bench", "--bench", "grisubal_grid_size"], stdout=open("grid/criterion.txt", "w"))
    subprocess.run(["cargo", "build", "--release", "--features=profiling", "--bin=grisubal"])
    for i in SIZE_RANGE:
        # avg runtimes per section (50 samples per grid size)
        with open(f"grid/{i:.1f}.csv", "w") as f:
            subprocess.run(["echo", CSV_HEADER], stdout=f)
        for _ in range(50):
            with open(f"grid/{i:.1f}.csv", "a") as f:
                subprocess.run([GRISUBAL, FILE_PATH, str(i), str(i)], stdout=f)
        # perf + flamegraph
        subprocess.run(
            ["perf", "record", "-o", f"grid/{i:.1f}.perf.data", "-F", PERF_FREQ, "--call-graph", "dwarf", GRISUBAL_PROF,
             FILE_PATH,
             FIXED_SIZE, FIXED_SIZE])
        subprocess.run(
            ["flamegraph", "--flamechart", "--perfdata", f"grid/{i:.1f}.perf.data", "-o", f"grid/{i:.1f}.svg"])
        # heaptrack
        subprocess.run(["heaptrack", "-o", f"grid/{i:.1f}", GRISUBAL, FILE_PATH, str(i), str(i)])
    return


def thread():
    print("Not yet implemented")
    return


# Main

def main():
    print("Run:")
    print("(0) all")
    print("(1) fixed-size profiling")
    print("(2) grid size scaling")
    print("(3) thread number scaling")
    print("(q) quit")

    choice = input("Enter your choice (0-3): ")

    if choice == "0":
        print("Running all benchmarks")
        fixed()
        grid()
        thread()
    elif choice == "1":
        print("Running fixed-size benchmarks")
        fixed()
    elif choice == "2":
        print("Running grid size scaling benchmarks")
        grid()
    elif choice == "3":
        print("Running thread number scaling benchmarks")
        subprocess.run(["df", "-h"])
    elif choice == "q":
        print("Quitting")
        sys.exit(0)
    else:
        print("Invalid option; Exiting.")
        sys.exit(1)
    print("Finished")
    sys.exit(0)


if __name__ == "__main__":
    main()
