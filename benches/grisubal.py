import subprocess
import sys

# Definitions

GRISUBAL = "./target/release/grisubal"
GRISUBAL_PROF = "./target/profiling/grisubal"
FILE_PATH = "./examples/shape.vtk"

FIXED_SIZE = "1."
SIZE_RANGE = [x / 10 for x in range(1, 11)]


# Benchmarks suite

def fixed():
    # avg runtime using criterion
    subprocess.run(["cargo", "bench", "--bench", "grisubal"], stdout=open("fixed_criterion.txt", "w"))
    # avg runtime per section (50 samples)
    for _ in range(50):
        with open("fixed.csv", "a") as f:
            subprocess.run([GRISUBAL, FILE_PATH, FIXED_SIZE, FIXED_SIZE], stdout=f)
    # perf + flamegraph

    # heaptrack
    subprocess.run(["heaptrack", "-o", "fixed", GRISUBAL_PROF, FILE_PATH, FIXED_SIZE, FIXED_SIZE])


def grid():
    # avg runtimes using criterion
    subprocess.run(["cargo", "bench", "--bench", "grisubal_grid_size"], stdout=open("grid_criterion.txt", "w"))
    # avg runtimes per section (50 samples per grid size)
    for i in SIZE_RANGE:
        for _ in range(50):
            with open(f"size{i:.1f}.csv", "a") as f:
                subprocess.run([GRISUBAL, FILE_PATH, str(i), str(i)], stdout=f)
    # perf + flamegraph
    # for i in [x / 10 for x in range(1, 11)]:

    # heaptrack
    for i in SIZE_RANGE:
        subprocess.run(["heaptrack", "-o", f"size{i:.1f}", GRISUBAL, FILE_PATH, str(i), str(i)])


def thread():
    print("Not yet implemented")


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


if __name__ == "__main__":
    main()
