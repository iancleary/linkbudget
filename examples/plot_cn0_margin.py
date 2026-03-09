#!/usr/bin/env python3
import csv
from pathlib import Path
import matplotlib.pyplot as plt

root = Path(__file__).resolve().parent
csv_path = root / "output" / "cn0_margin_samples.csv"
out_png = root / "output" / "cn0_margin_plots.png"

vals = []
with csv_path.open() as f:
    r = csv.DictReader(f)
    for row in r:
        vals.append(float(row["margin_db"]))

vals_sorted = sorted(vals)
n = len(vals_sorted)
cdf_y = [(i + 1) / n for i in range(n)]

fig, axes = plt.subplots(1, 2, figsize=(12, 4.5))
axes[0].hist(vals, bins=80)
axes[0].axvline(0.0, linestyle="--")
axes[0].set_title("C/N0 Margin Distribution")
axes[0].set_xlabel("Margin vs target (dB)")
axes[0].set_ylabel("Count")

axes[1].plot(vals_sorted, cdf_y)
axes[1].axvline(0.0, linestyle="--")
axes[1].set_title("C/N0 Margin CDF")
axes[1].set_xlabel("Margin vs target (dB)")
axes[1].set_ylabel("P(Margin ≤ x)")

fig.tight_layout()
fig.savefig(out_png, dpi=150)
print(f"Saved {out_png}")
