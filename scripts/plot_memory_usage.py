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
        oax.pie(totals, 
            explode=explode, 
            wedgeprops={"edgecolor":"white"}, 
            autopct='%1.1f%%')
        plt.legend(
            title="Categories", 
            ncol=1,
            labels= category_labels, 
            loc="center right", 
            bbox_to_anchor=(1.3, 0.5), 
            draggable=True)
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
        inner_colors = cmap([1, 2, 3, 5, 6, 7, 9, 10, 13, 14, 15])

        dax.pie(totals, 
            radius=1, 
            colors=outer_colors, 
            autopct='%1.1f%%',
            pctdistance=1.25,
            explode=explode,
            wedgeprops=dict(width=size, edgecolor='w'))

        dax.pie(vals, 
            radius=1-size, 
            colors=inner_colors, 
            labels = beta_labels + embed_labels + geometry_labels + others_labels,
            labeldistance=.65,
            textprops={'size': 'xx-small'},
            explode=[0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02, 0.02],
            wedgeprops=dict(width=size, edgecolor='w'))

        plt.title("Memory Usage: Detailed")
        plt.legend(
            title="Categories", 
            ncol=1,
            labels= category_labels, 
            loc="center right", 
            bbox_to_anchor=(1.3, 0.5), 
            draggable=True)
        if show:
            plt.show()
        else: 
            plt.savefig(save_file + "_detailed.svg")
    
    if all_cats:
        cmap = plt.colormaps["tab20c"]
        pie_colors = cmap([0, 4, 8, 12])
        beta_colors = cmap([1, 2, 3])
        embed_colors = cmap([5, 6, 7])
        geometry_colors = cmap([9, 10])
        others_colors = cmap([13, 14, 15])

        # --- beta chart ---
        beta_afig, (beta_aax1, beta_aax2) = plt.subplots(1, 2, figsize=(9, 5))
        beta_afig.subplots_adjust(wspace=0)

        beta_aax1.pie(totals, 
            labels=category_labels,
            explode=explode, 
            colors=pie_colors,
            wedgeprops={"edgecolor":"white"}, 
            autopct='%1.1f%%')
        plt.title("Memory Usage: Beta functions")

        beta_ratios = [b / totals[0] for b in beta]
        bottom = 1
        width = .2

        for j, (height, label) in enumerate(reversed([*zip(beta_ratios, beta_labels)])):
            bottom -= height
            bc = beta_aax2.bar(0, height, width, bottom=bottom, color=beta_colors, label=label,
                        alpha=0.1 + 0.25 * j)
            beta_aax2.bar_label(bc, labels=[f"{height:.0%}"], label_type='center')

        beta_aax2.set_title("Beta functions")
        beta_aax2.legend()
        beta_aax2.axis('off')
        beta_aax2.set_xlim(- 2.5 * width, 2.5 * width)

        if show:
            plt.show()
        else: 
            plt.savefig(save_file + "_beta.svg")

        # --- embed chart ---
        embed_afig, (embed_aax1, embed_aax2) = plt.subplots(1, 2, figsize=(9, 5))
        embed_afig.subplots_adjust(wspace=0)

        embed_aax1.pie(totals, 
            labels=category_labels,
            explode=explode, 
            colors=pie_colors,
            wedgeprops={"edgecolor":"white"}, 
            autopct='%1.1f%%')
        plt.title("Memory Usage: Embedded data")

        embed_ratios = [e / totals[1] for e in embed]
        bottom = 1
        width = .2

        for j, (height, label) in enumerate(reversed([*zip(embed_ratios, embed_labels)])):
            bottom -= height
            bc = embed_aax2.bar(0, height, width, bottom=bottom, color=embed_colors, label=label,
                        alpha=0.1 + 0.25 * j)
            embed_aax2.bar_label(bc, labels=[f"{height:.0%}"], label_type='center')

        embed_aax2.set_title("Embedded data")
        embed_aax2.legend()
        embed_aax2.axis('off')
        embed_aax2.set_xlim(- 2.5 * width, 2.5 * width)

        if show:
            plt.show()
        else: 
            plt.savefig(save_file + "_embed.svg")

        # --- geometry chart ---
        geometry_afig, (geometry_aax1, geometry_aax2) = plt.subplots(1, 2, figsize=(9, 5))
        geometry_afig.subplots_adjust(wspace=0)

        geometry_aax1.pie(totals, 
            labels=category_labels,
            explode=explode, 
            colors=pie_colors,
            wedgeprops={"edgecolor":"white"}, 
            autopct='%1.1f%%')
        plt.title("Memory Usage: Geometrical data")

        geometry_ratios = [g / totals[2] for g in geometry]
        bottom = 1
        width = .2

        for j, (height, label) in enumerate(reversed([*zip(geometry_ratios, geometry_labels)])):
            bottom -= height
            bc = geometry_aax2.bar(0, height, width, bottom=bottom, color=geometry_colors, label=label,
                        alpha=0.1 + 0.25 * j)
            geometry_aax2.bar_label(bc, labels=[f"{height:.0%}"], label_type='center')

        geometry_aax2.set_title("Geometrical data")
        geometry_aax2.legend()
        geometry_aax2.axis('off')
        geometry_aax2.set_xlim(- 2.5 * width, 2.5 * width)

        if show:
            plt.show()
        else: 
            plt.savefig(save_file + "_geometry.svg")

        # --- others chart ---
        others_afig, (others_aax1, others_aax2) = plt.subplots(1, 2, figsize=(9, 5))
        others_afig.subplots_adjust(wspace=0)

        others_aax1.pie(totals, 
            labels=category_labels,
            explode=explode, 
            colors=pie_colors,
            wedgeprops={"edgecolor":"white"}, 
            autopct='%1.1f%%')
        plt.title("Memory Usage: Miscellaneous data")

        others_ratios = [o / totals[3] for o in others]
        bottom = 1
        width = .2

        for j, (height, label) in enumerate(reversed([*zip(others_ratios, others_labels)])):
            bottom -= height
            bc = others_aax2.bar(0, height, width, bottom=bottom, color=others_colors, label=label,
                        alpha=0.1 + 0.25 * j)
            others_aax2.bar_label(bc, labels=[f"{height:.0%}"], label_type='center')

        others_aax2.set_title("Miscellaneous data")
        others_aax2.legend()
        others_aax2.axis('off')
        others_aax2.set_xlim(- 2.5 * width, 2.5 * width)

        if show:
            plt.show()
        else: 
            plt.savefig(save_file + "_others.svg")

if __name__ == "__main__":
    run()