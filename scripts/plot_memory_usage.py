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

    beta, embed, geometry, others, totals = parseDataFromFile(args[0])
    print(totals)


if __name__ == "__main__":
    run()