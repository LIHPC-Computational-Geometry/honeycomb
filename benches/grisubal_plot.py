import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from matplotlib import font_manager

# Read the CSV file
df = pd.read_csv('fixed/sections.csv')

if 'BuildMeshTot' in df.columns:
    df = df.drop('BuildMeshTot', axis=1)

# Calculate the mean time for each section
mean_times = df.mean()

# Separate BuildMesh sections and other sections
buildmesh_columns = [col for col in mean_times.index if col.startswith('BuildMesh')]
other_columns = [col for col in mean_times.index if not col.startswith('BuildMesh')]

# Sort BuildMesh sections and other sections separately
sorted_buildmesh = mean_times[buildmesh_columns].sort_values(ascending=False)
sorted_others = mean_times[other_columns].sort_values(ascending=False)

# Combine sorted BuildMesh and other sections
sorted_times = pd.concat([sorted_buildmesh, sorted_others])

# Create a color map: shades of blue for BuildMesh, other colors for the rest
num_buildmesh = len(buildmesh_columns)
num_others = len(other_columns)

blues = plt.cm.Blues(np.linspace(0.6, 1, num_buildmesh))
other_colors = plt.cm.Set3(np.linspace(0, 0.6, num_others))

colors = list(blues) + list(other_colors)

# Create a pie chart
plt.figure(figsize=(16, 12))

wedges, texts, autotexts = plt.pie(sorted_times.values,
                                   colors=colors,
                                   autopct=lambda pct: f'{pct:.1f}%',
                                   pctdistance=1.05,  # Move percentage labels outside
                                   startangle=90,  # Start at top
                                   wedgeprops=dict(width=0.5, edgecolor='white'))  # Make it a donut chart

# Remove the texts (labels) from the pie
for text in texts:
    text.set_text('')

# Enhance the appearance
plt.title('Relative Time Spent in Each Section', fontsize=16)
plt.axis('equal')  # Equal aspect ratio ensures that pie is drawn as a circle

# Adjust percentage text size and make them bold
plt.setp(autotexts, size=9, weight="bold")


# Format time values for legend
def format_time(ns):
    if ns >= 1e6:
        return f'{ns / 1e6:.2f}'.rjust(6) + ' ms'
    elif ns >= 1e3:
        return f'{ns / 1e3:.2f}'.rjust(6) + ' µs'
    else:
        return f'{ns:.2f}'.rjust(6) + ' ns'


max_name_length = max(len(name) for name in sorted_times.index)

# Create legend labels with section names and time values, using tab for alignment
legend_labels = [f'{name.ljust(max_name_length)} : {format_time(time)}' for name, time in sorted_times.items()]

# Find a monospace font
monospace_font = font_manager.FontProperties(family='monospace')

# Add a legend with formatted time values
legend = plt.legend(wedges, legend_labels, title="Sections", loc="center left",
                    bbox_to_anchor=(1, 0, 0.5, 1), fontsize=9, prop=monospace_font)

# Adjust layout and save the plot
plt.tight_layout()
plt.savefig('time_spent_pie_chart_grouped.svg', bbox_inches='tight', dpi=300)

print("Pie chart has been saved as 'time_spent_pie_chart_grouped.svg'")
