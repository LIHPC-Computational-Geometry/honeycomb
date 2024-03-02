import sys
import matplotlib.pyplot as plt
import csv

def parseDataFromFile(filename: str):
    beta = []
    embed = []
    geometry = []
    others = []
    totals = []

    with open(filename, newline = '') as csvfile:
        rdr = csv.reader(csvfile, delimiter = ',')
        for row in rdr:
            print(row)
            if "beta_" in row[0]:
                if "total" in row[0]:
                    totals.append(int(row[1]))
                else:
                    beta.append(int(row[1]))
            elif "embed_" in row[0]:
                if "total" in row[0]:
                    totals.append(int(row[1]))
                else:
                    embed.append(int(row[1]))
            elif "geometry_" in row[0]:
                if "total" in row[0]:
                    totals.append(int(row[1]))
                else:
                    geometry.append(int(row[1]))
            elif "others" in row[0]:
                if "total" in row[0]:
                    totals.append(int(row[1]))
                else:
                    others.append(int(row[1]))
    return (beta, embed, geometry, others, totals)

def run():
    opts = [opt for opt in sys.argv[1:] if opt.startswith("-")]
    args = [arg for arg in sys.argv[1:] if not arg.startswith("-")]
    
    if len(args) > 1:
        print("W: Multiple arguments provided when only one is needed")
        print("Usage:")
        print("$ python3 plot_memory_usage.py <CSVFILE> <OPTIONS>")

    filename = args[0]

    overview = False
    detailed = False
    all = False

    if ".csv" not in filename:
        print("W: Specified file may not have the correct format")

    if "--help" in opts or "-h" in opts: # --help / -h -- prints a help message
        print("Usage:")
        print("$ python3 plot_memory_usage.py <CSVFILE> <OPTIONS>")
        print("Available options:")
        print("  --help       -h  --  prints this message")
        print("  --overview   -o  --  generates a plot exclusively using total category values")
        print("  --detailed   -d  --  generates a plot using all values of each category")
        print("  --all        -a  --  generates a zoomed-in plot for each category")
        return
    else:
        if "--overview" in opts or "-o" in opts:
            overview = True
        if "--detailed" in opts or "-d" in opts:
            detailed = True
        if "--all" in opts or "-a" in opts:
            all = True
    
    beta, embed, geometry, others, totals = parseDataFromFile(args[0])
    print(totals)


if __name__ == "__main__":
    run()