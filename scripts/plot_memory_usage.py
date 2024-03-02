import sys
import matplotlib.pyplot as plt
import csv

def parseDataFromFile(filename: str):
    beta = []
    embed = []
    geometry = []
    others = []

    with open(filename, newline = '') as csvfile:
        rdr = csv.reader(csvfile, delimiter = ',')
        for row in rdr:
            print(row)

def run():
    opts = [opt for opt in sys.argv[1:] if opt.startswith("-")]
    args = [arg for arg in sys.argv[1:] if not arg.startswith("-")]

    parseDataFromFile(args[0])


if __name__ == "__main__":
    run()