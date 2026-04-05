from collections import Counter
from dataclasses import dataclass

from PIL import Image
from colormath.color_conversions import convert_color
from colormath.color_objects import sRGBColor, LabColor
from src.texture import Block

type Lab = tuple[float, float, float]


@dataclass
class BlockStats:
    b: Block  # block
    a: Lab  # avg
    w: list[tuple[float, float, float, float]]  # weights


def _rgb_to_lab(r: int, g: int, b: int) -> Lab:
    rgb = sRGBColor(r, g, b, is_upscaled=True)
    lab: LabColor = convert_color(rgb, LabColor)
    return lab.lab_l, lab.lab_a, lab.lab_b


def _quantise_lab(lab: Lab, step: float = 1.0) -> Lab:
    l, a, b = lab
    return (
        round(l / step) * step,
        round(a / step) * step,
        round(b / step) * step,
    )


def _colour_frequency(image: Image.Image) -> list[tuple[Lab, int]]:
    """Returns list of ((l, a, b), count)"""
    # count in RGB
    rgba_freq = Counter(image.get_flattened_data())

    # convert unique colours only
    freq = Counter()

    for (r, g, b, _), count in rgba_freq.items():
        lab = _rgb_to_lab(r, g, b)
        lab_q = _quantise_lab(lab, step=1.0)
        freq[lab_q] += count

    return list(freq.items())


def _normalise(freq: list[tuple[Lab, int]]) -> list[tuple[float, float, float, float]]:
    total = sum(count for _, count in freq)
    if total == 0:
        return []

    return [
        (l, a, b, count / total)
        for (l, a, b), count in freq
    ]


def _average_lab(freq: list[tuple[Lab, int]]) -> Lab:
    total = sum(count for _, count in freq)
    if total == 0:
        return 0.0, 0.0, 0.0

    l = sum(l * c for (l, _, _), c in freq) / total
    a = sum(a * c for (_, a, _), c in freq) / total
    b = sum(b * c for (_, _, b), c in freq) / total

    return l, a, b


def compute_stats(textures: dict[Block, Image.Image]) -> list[BlockStats]:
    stats = []

    for block, texture in textures.items():
        freq = _colour_frequency(texture)
        avg = _average_lab(freq)
        weights = _normalise(freq)

        stats.append(BlockStats(block, avg, weights))

    return stats
