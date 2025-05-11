import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
from sympy import symbols, Eq, solve

df = pd.read_excel('polygon_intersection_results.xlsx')

row1 = df.iloc[0]
row2 = df.iloc[1]

n1, y1 = float(row1['n']), float(row1['probability'])
n2, y2 = float(row2['n']), float(row2['probability'])

a, b = symbols('a b')
eq1 = Eq(1/3 + a/(n1 + b), y1)
eq2 = Eq(1/3 + a/(n2 + b), y2)
sol = solve((eq1, eq2), (a, b))
a_exact, b_exact = sol[a], sol[b]

print(f"Exact equation: y = 1/3 + ({a_exact})/(n + ({b_exact}))")
print(f"Decimal form: y = 1/3 + {float(a_exact):.8f}/(n + {float(b_exact):.8f})")

n_vals = np.linspace(min(n1, n2), max(df['n']), 500)
y_exact = 1/3 + float(a_exact)/(n_vals + float(b_exact))

plt.figure(figsize=(10, 6))
plt.plot(n_vals, y_exact, 'r-', label=f'Exact: $y = \\frac{{1}}{{3}} + \\frac{{{float(a_exact):.4f}}}{{n + {float(b_exact):.4f}}}$')
plt.scatter([n1, n2], [y1, y2], color='blue', zorder=5, label='Used points')
plt.axhline(y=1/3, color='green', linestyle='--', alpha=0.7, label='Asymptote (y=1/3)')
plt.xlabel('Number of Sides (n)')
plt.ylabel('Probability')
plt.title('Exact Equation Passing Through Two Data Points')
plt.legend()
plt.grid(True, alpha=0.3)
plt.tight_layout()
plt.savefig('exact_equation_from_excel.png', dpi=300)
plt.show()