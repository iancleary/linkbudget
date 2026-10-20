"""Independently check the generated CSV/report using only the Python stdlib."""

import csv
import math
import statistics
from pathlib import Path


def main() -> None:
    """Check consumer-visible statistics, not statistical coverage or field validity."""
    output = Path(__file__).resolve().parent / "output"
    with (output / "receiver_ebno_margin_samples.csv").open() as samples:
        reader = csv.DictReader(samples)
        assert reader.fieldnames == ["eb_no_margin_db"]
        values = [float(row["eb_no_margin_db"]) for row in reader]
    report = dict(
        line.split("=", 1)
        for line in (output / "receiver_ebno_margin_summary.txt").read_text().splitlines()
    )
    n = len(values)
    assert n == int(report["trials"]) == 50_000
    assert values == sorted(values) and all(math.isfinite(v) for v in values)
    failures = sum(v < 0 for v in values)
    assert failures == int(report["failures"])
    p = failures / n
    assert math.isclose(p, float(report["failure_probability"]), rel_tol=0.0, abs_tol=1e-15)
    assert math.isclose(statistics.fmean(values), float(report["mean_margin_db"]), rel_tol=0.0, abs_tol=1e-12)
    quantiles = statistics.quantiles(values, n=100, method="inclusive")
    for percentile in (5, 50, 95):
        assert math.isclose(
            quantiles[percentile - 1],
            float(report[f"p{percentile:02}_margin_db"]),
            rel_tol=0.0,
            abs_tol=1e-12,
        )
    z = statistics.NormalDist().inv_cdf(0.975)
    # Solve the score-test quadratic: N(p_hat-p)^2 = z^2*p*(1-p).
    a, b, c = n + z * z, -(2 * failures + z * z), failures * failures / n
    discriminant = math.sqrt(b * b - 4 * a * c)
    lower, upper = (-b - discriminant) / (2 * a), (-b + discriminant) / (2 * a)
    for name, expected in (("lower", lower), ("upper", upper)):
        assert math.isclose(
            expected, float(report[f"failure_wilson_two_sided_95_{name}"]), rel_tol=0.0, abs_tol=1e-14
        )
    zero_upper = -math.expm1(math.log(0.05) / n)
    assert math.isclose(zero_upper, 5.99128506245549e-5, rel_tol=0.0, abs_tol=1e-17)
    if failures == 0:
        assert math.isclose(
            zero_upper, float(report["zero_failure_exact_one_sided_95_upper"]), rel_tol=0.0, abs_tol=1e-17
        )
    # Independent ideal QPSK threshold from the stdlib erfc, not crate output.
    lo, hi = 0.0, 20.0
    for _ in range(100):
        mid = (lo + hi) / 2
        if 0.5 * math.erfc(math.sqrt(10 ** (mid / 10))) > 1e-6:
            lo = mid
        else:
            hi = mid
    assert abs(float(report["required_eb_no_db"]) - (lo + hi) / 2) < 0.001
    assert int(report["seed"]) == 99
    assert float(report["information_rate_bps"]) == 10e6
    assert float(report["target_ber"]) == 1e-6
    print(f"Verified N={n}, failures={failures}, p={p:.8g}, Wilson95=[{lower:.12g}, {upper:.12g}]")
    print(f"Margins p05/p50/p95: {quantiles[4]:.9f}, {quantiles[49]:.9f}, {quantiles[94]:.9f} dB")
    print(f"Zero-failure one-sided exact95 upper at N={n}: {zero_upper:.17g}")


if __name__ == "__main__":
    main()
