#!/usr/bin/env python3
import csv
from pathlib import Path
import matplotlib.pyplot as plt

root = Path(__file__).resolve().parent
csv_path = root / "output" / "cn0_margin_samples.csv"
out_png = root / "output" / "cn0_margin_plots.png"
vals = [float(r["margin_db"]) for r in csv.DictReader(csv_path.open())]
vals_sorted = sorted(vals)
n = len(vals_sorted)

fig, axes = plt.subplots(1, 2, figsize=(12, 4.5))
axes[0].hist(vals, bins=80)
axes[0].axvline(0.0, linestyle="--")
axes[0].set_title("C/N0 Margin Distribution")
axes[1].plot(vals_sorted, [(i + 1) / n for i in range(n)])
axes[1].axvline(0.0, linestyle="--")
axes[1].set_title("C/N0 Margin CDF")
fig.tight_layout(); fig.savefig(out_png, dpi=150)
print(f"Saved {out_png}")
