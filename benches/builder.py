import subprocess
import sys

# Definitions

BUILDER_PROF = "./target/profiling/builder"

FIXED_SIZE = "128"
SIZE_RANGE = [2 ** n for n in range(7, 13)]  # 128*128 to 8192*8192

PERF_FREQ = "997"


# Benchmarks suite

def fixed():
    # avg runtime using criterion
    subprocess.run(["mkdir", "fixed"])
    subprocess.run(["cargo", "bench", "--bench", "builder"], stdout=open("fixed/criterion.txt", "w"))
    subprocess.run(["cargo", "build", "--profile=profiling", "--bin=builder"])
    # perf + flamegraph
    subprocess.run(
        ["perf", "record", "-o", "fixed/perf.data", "-F", PERF_FREQ, "--call-graph", "dwarf", BUILDER_PROF,
         FIXED_SIZE, FIXED_SIZE])
    subprocess.run(["flamegraph", "--flamechart", "--perfdata", "fixed/perf.data", "-o", "fixed/fg.svg"])
    # heaptrack
    subprocess.run(["heaptrack", "-o", "fixed/ht", BUILDER_PROF, FIXED_SIZE, FIXED_SIZE])
    return


def size():
    # avg runtimes using criterion
    subprocess.run(["mkdir", "size"])
    subprocess.run(["cargo", "bench", "--bench", "builder-grid-size"], stdout=open("size/criterion.txt", "w"))
    subprocess.run(["cargo", "build", "--profile=profiling", "--bin=builder"])
    for i in SIZE_RANGE:
        # perf + flamegraph
        subprocess.run(
            ["perf", "record", "-o", f"size/{i}.perf.data", "-F", PERF_FREQ, "--call-graph", "dwarf", BUILDER_PROF,
             FIXED_SIZE, FIXED_SIZE])
        subprocess.run(
            ["flamegraph", "--flamechart", "--perfdata", f"size/{i}.perf.data", "-o", f"size/{i:.1f}.svg"])
        # heaptrack
        subprocess.run(["heaptrack", "-o", f"size/{i}", BUILDER_PROF, str(i), str(i)])
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
        size()
        thread()
    elif choice == "1":
        print("Running fixed-size benchmarks")
        fixed()
    elif choice == "2":
        print("Running grid size scaling benchmarks")
        size()
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
