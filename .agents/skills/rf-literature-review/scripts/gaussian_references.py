"""Print the fixed Gaussian reference set, independently of the Rust kernel.

Uses only stdlib Decimal: Machin pi, erf Taylor summation, and argument
bisection. Cancellation is intentional. Agreement at two precisions is
empirical evidence for this finite set, not a rigorous arbitrary-tail bound.
No inputs, network access, or file writes; output goes only to stdout.
"""

from collections.abc import Callable
from decimal import Decimal, localcontext


ERFC_INPUTS = (
    "0", "0.125", "0.5", "1", "1.499999999999", "1.5",
    "1.500000000001", "2", "4", "8", "10", "20", "26",
    "27", "27.2", "27.25", "27.3",
)
BER_INPUTS = ((0, 2), (10, 2), (20, 2), (30, 2), (10, 8), (15, 16), (20, 64))


def _atan_inverse(n: int) -> Decimal:
    x = Decimal(1) / n
    power = total = x
    for i in range(1, 10_000):
        power *= -x * x
        previous = total
        total += power / (2 * i + 1)
        if total == previous:
            return total
    raise ArithmeticError(f"Machin atan(1/{n}): 10000-term cap exceeded")


def _bisect_argument(
    above_target: Callable[[Decimal], bool], low: Decimal, high: Decimal
) -> Decimal:
    # A fixed 65 steps reproduces the original generator's argument resolution.
    if not above_target(low) or above_target(high):
        raise ArithmeticError("reference target is not bracketed")
    for _ in range(65):
        middle = (low + high) / 2
        if above_target(middle):
            low = middle
        else:
            high = middle
    return (low + high) / 2


def _references(precision: int) -> dict[str, float]:
    with localcontext() as context:
        context.prec = precision
        pi = 16 * _atan_inverse(5) - 4 * _atan_inverse(239)
        factor = 2 / pi.sqrt()
        sqrt_two = Decimal(2).sqrt()

        def erfc(x: Decimal) -> Decimal:
            if not Decimal(0) <= x <= Decimal(33):
                raise ArithmeticError("erfc reference argument outside [0, 33]")
            power = total = x
            for n in range(1, 30_000):
                power *= -x * x / n
                previous = total
                total += power / (2 * n + 1)
                if previous == total:
                    result = 1 - factor * total
                    if not Decimal(0) < result <= Decimal(1):
                        raise ArithmeticError("erfc cancellation exhausted precision")
                    return result
            raise ArithmeticError(f"erf({x}): 30000-term cap exceeded")

        values: dict[str, float] = {}
        for text in ERFC_INPUTS:
            # Match the actual binary64 argument, not its decimal spelling.
            values[f"erfc {text}"] = float(erfc(Decimal.from_float(float(text))))

        for db, order in BER_INPUTS:
            gamma = (Decimal(db) / 10 * Decimal(10).ln()).exp()
            if order == 2:
                probability = erfc(gamma.sqrt()) / 2
            elif order == 8:
                # Exact sin(pi/8), independent of a library trig routine.
                sine = (2 - sqrt_two).sqrt() / 2
                probability = erfc((3 * gamma).sqrt() * sine) / 3
            else:
                bits = Decimal(order).ln() / Decimal(2).ln()
                argument = (3 * bits * gamma / (2 * (order - 1))).sqrt()
                probability = (
                    4 / bits * (1 - 1 / Decimal(order).sqrt()) * erfc(argument) / 2
                )
            values[f"ber {db} {order}"] = float(probability)
            values[f"logber {db} {order}"] = float(probability.ln())

        for text in ("0.000001", "1e-100", "1e-300"):
            target = Decimal(text)
            argument = _bisect_argument(
                lambda x: erfc(x) / 2 > target, Decimal(0), Decimal(30)
            )
            values[f"threshold {text}"] = float(20 * argument.log10())

        argument = _bisect_argument(
            lambda x: (erfc(x) / 2).ln() > -1000, Decimal(30), Decimal(33)
        )
        values["threshold log-1000"] = float(20 * argument.log10())

        for text in ("0.1", "1e-100", "1e-300", "minsubnormal"):
            target = (
                Decimal.from_float(float.fromhex("0x0.0000000000001p-1022"))
                if text == "minsubnormal"
                else Decimal(text)
            )
            argument = _bisect_argument(
                lambda x: erfc(x / sqrt_two) / 2 > target,
                Decimal(0),
                Decimal(40),
            )
            values[f"inverseq {text}"] = float(argument)
        return values


def main() -> None:
    """Assert rounded binary64 stability at 1200/1600 digits, then print."""
    baseline = _references(1200)
    higher_precision = _references(1600)
    for label, value in baseline.items():
        if value.hex() != higher_precision[label].hex():
            raise ArithmeticError(
                f"precision instability for {label}: "
                f"{value!r} != {higher_precision[label]!r}"
            )
    print("Stable binary64 references at 1200 and 1600 Decimal digits:")
    for label, value in baseline.items():
        print(f"{label}: {value!r}")


if __name__ == "__main__":
    main()
