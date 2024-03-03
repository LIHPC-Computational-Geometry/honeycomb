import csv
import matplotlib.pyplot as plt
import numpy as np
import sys

def parseCommandLine():
    opts = [opt for opt in sys.argv[1:] if opt.startswith("-")]
    args = [arg for arg in sys.argv[1:] if not arg.startswith("-")]
    
    if len(args) > 1:
        print("W: Multiple arguments provided when only one is needed")
        print("Usage:")
        print("$ python3 plot_memory_usage.py <CSVFILE> <OPTIONS>")

    filename = args[0]

    show = False
    overview = False
    detailed = False
    all_cats = False

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
        exit(0)
    
    if "--show" in opts:
        show = True
    if "--overview" in opts or "-o" in opts:
        overview = True
    if "--detailed" in opts or "-d" in opts:
        detailed = True
    if "--all" in opts or "-a" in opts:
        all_cats = True
    
    return (filename, show, overview, detailed, all_cats)

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
    filename, show, overview, detailed, all_cats = parseCommandLine()
    beta, embed, geometry, others, totals = parseDataFromFile(filename)

    category_labels = ["Beta", "Embed", "Geometry", "Others"]
    beta_labels = ["β0", "β1", "β2"]
    embed_labels = ["vertexIDs", "faceIDs", "marks"]
    geometry_labels = ["vertex", "face"]
    others_labels = ["freedarts", "freevertices", "counters"]
    explode = [0.02, 0.02, 0.02, 0.02]

    save_file = filename[0:-4]

    if overview:
        ofig, oax = plt.subplots()
        oax.pie(totals, explode=explode, wedgeprops={"edgecolor":"white"}, autopct='%1.1f%%')
        plt.legend(
            title="Categories", 
            labels= category_labels, 
            loc="center right", 
            bbox_to_anchor=(1.3, 0.5), 
            ncol=1)
        plt.title("Memory Usage: Overview")
        if show:
            plt.show()
        else: 
            plt.savefig(save_file + "_overview.svg")

    if detailed:
        dfig, dax = plt.subplots()

        size = 0.3
        vals = beta + embed + geometry + others

        cmap = plt.colormaps["tab20c"]
        outer_colors = cmap([0, 4, 8, 12])
        inner_colors = cmap([1, 2, 3, 5, 6, 7, 9, 10, 11, 12, 13])

        dax.pie(totals, radius=1, colors=outer_colors, explode=explode,
            wedgeprops=dict(width=size, edgecolor='w'))

        dax.pie(vals, radius=1-size, colors=inner_colors, explode=[0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02],
            wedgeprops=dict(width=size, edgecolor='w'))

        plt.title("Memory Usage: Overview")
        plt.legend(
            title="Categories", 
            labels= category_labels + beta_labels + embed_labels + geometry_labels + others_labels, 
            loc="center right", 
            bbox_to_anchor=(1.3, 0.5), 
            ncol=1)
        if show:
            plt.show()
        else: 
            plt.savefig(save_file + "_detailed.svg")
    
    if all_cats:
        dfig, dax = plt.subplots()

if __name__ == "__main__":
    run()