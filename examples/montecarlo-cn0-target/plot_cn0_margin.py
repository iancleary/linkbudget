#!/usr/bin/env python3
import csv
from pathlib import Path
import matplotlib.pyplot as plt

root = Path(__file__).resolve().parent
csv_path = root / "output" / "receiver_ebno_margin_samples.csv"
out_png = root / "output" / "receiver_ebno_margin_plots.png"
with csv_path.open() as samples:
    vals = [float(r["eb_no_margin_db"]) for r in csv.DictReader(samples)]
vals_sorted = sorted(vals)
n = len(vals_sorted)

fig, axes = plt.subplots(1, 2, figsize=(12, 4.5))
axes[0].hist(vals, bins=80)
axes[0].axvline(0.0, linestyle="--")
axes[0].set_title("Illustrative Receiver Eb/N0 Margin Distribution")
axes[0].set_xlabel("Eb/N0 margin vs QPSK BER 1e-6 (dB)")
axes[0].set_ylabel("Count")

axes[1].plot(vals_sorted, [(i + 1) / n for i in range(n)])
axes[1].axvline(0.0, linestyle="--")
axes[1].set_title("Illustrative Receiver Eb/N0 Margin CDF")
axes[1].set_xlabel("Eb/N0 margin vs QPSK BER 1e-6 (dB)")
axes[1].set_ylabel("P(Margin ≤ x)")

fig.tight_layout()
fig.savefig(out_png, dpi=150)
print(f"Saved {out_png}")
