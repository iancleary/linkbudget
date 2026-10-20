"""Independent second-order complex-AWGN references; stdlib only, no file writes.

Decimal erf Taylor series + Machin pi and bisection, not the Rust tail kernel.
Checks rounded stability at 80/120 digits; not a rigorous error bound.
Run from the checkout root with uv run --offline --no-project python <this file>.
"""

from decimal import Decimal, localcontext

from gaussian_references import _atan_inverse, _bisect_argument


CASES = (
    ("1", 1024, "0.001"),
    ("1", 1375, "0.001"),
    ("0.1", 1000, "0.1"),
    ("10", 200, "0.000001"),
    ("100", 100, "0.000000001"),
    ("1", 100, "0.1"),
)


def references(precision: int) -> dict[str, float]:
    with localcontext() as context:
        context.prec = precision
        pi = 16 * _atan_inverse(5) - 4 * _atan_inverse(239)
        factor = 2 / pi.sqrt()
        sqrt_two = Decimal(2).sqrt()
        log_two = Decimal(2).ln()

        def q(x: Decimal) -> Decimal:
            x /= sqrt_two
            power = total = x
            for i in range(1, 10_000):
                power *= -x * x / i
                previous = total
                total += power / (2 * i + 1)
                if previous == total:
                    return (1 - factor * total) / 2
            raise ArithmeticError("erf series cap exceeded")

        values = {}
        for rho_text, n, epsilon_text in CASES:
            rho, epsilon = Decimal(rho_text), Decimal(epsilon_text)
            z = _bisect_argument(lambda x: q(x) > epsilon, Decimal(0), Decimal(10))
            capacity = (1 + rho).ln() / log_two
            dispersion = (1 - (1 + rho) ** -2) / log_two**2
            rate = capacity - (dispersion / n).sqrt() * z
            label = f"rho={rho_text} n={n} epsilon={epsilon_text}"
            values[label] = float(rate)
            values[f"inverseq {epsilon_text}"] = float(z)
            if n == 1375:
                values["Table I normalized with real-use correction"] = float(
                    rate + Decimal(2750).ln() / log_two / 2750
                )
        # Independent SI/Friis control for the documented public LinkBudget.
        d = Decimal
        received_w = d(10) ** (d(10 + 35 + 40 - 3 - 30) / 10) * (
            d(299792458) / (4 * pi * d("20e9") * d("550e3"))
        ) ** 2
        c_no = received_w / (d("1.380649e-23") * 200)
        rho = c_no / d("20e6")
        z = _bisect_argument(lambda x: q(x) > d("0.001"), d(0), d(10))
        capacity = (1 + rho).ln() / log_two
        dispersion = (1 - (1 + rho) ** -2) / log_two**2
        values["link C/N0 dB-Hz"] = float(10 * c_no.log10())
        values["link channel rho"] = float(rho)
        values["link Shannon bit/s"] = float(d("20e6") * capacity)
        values["link second-order bits/complex-use"] = float(
            capacity - (dispersion / 1024).sqrt() * z
        )
        return values


def main() -> None:
    baseline = references(80)
    higher = references(120)
    if baseline != higher:
        raise ArithmeticError("80/120-digit rounded references disagree")
    print("Stable binary64 references at 80/120 Decimal digits:")
    for label, value in baseline.items():
        print(f"{label}: {value!r}")


if __name__ == "__main__":
    main()
