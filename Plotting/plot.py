import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import matplotlib as mpl

plt.style.use('seaborn-v0_8-whitegrid')
mpl.rcParams['font.family'] = 'serif'

df = pd.read_excel('polygon_intersection_results.xlsx')

fig, ax = plt.subplots(figsize=(12, 7))

ax.plot(df['n'], df['probability'], '-', color='#1f77b4', lw=2.5, label='Probability')

ax.axhline(y=1/3, color='green', linestyle='--', alpha=0.7, label='Asymptote (y=1/3)')

error_multiplier = 1

lower_bound = df['probability'] - (df['probability'] - df['ci_lower']) * error_multiplier
upper_bound = df['probability'] + (df['ci_upper'] - df['probability']) * error_multiplier

lower_bound = lower_bound.clip(lower=0)
upper_bound = upper_bound.clip(upper=1)

ax.fill_between(df['n'], lower_bound, upper_bound,
                color='#1f77b4', alpha=0.2,
                label=f'95% Confidence Interval (×{error_multiplier} for visibility)')

ax.set_title('Probability of Line Segment Intersection in Regular Polygons', fontsize=18)
ax.set_xlabel('Number of Sides (n)', fontsize=16)
ax.set_ylabel('Probability', fontsize=16)

ax.grid(True, linestyle='--', alpha=0.7)
ax.spines['top'].set_visible(False)
ax.spines['right'].set_visible(False)
ax.tick_params(axis='both', which='major', labelsize=12)

ax.legend(fontsize=14, loc='best')

ax.set_ylim(0, 1)

info_text = "Monte Carlo Simulation\n1,000,000,000 iterations per n"
props = dict(boxstyle='round', facecolor='wheat', alpha=0.3)
ax.text(0.05, 0.95, info_text, transform=ax.transAxes, fontsize=12,
        verticalalignment='top', bbox=props)

plt.tight_layout()
plt.savefig('polygon_intersection_probability.png', dpi=300, bbox_inches='tight')
plt.savefig('polygon_intersection_probability.pdf', bbox_inches='tight')
plt.show()