from collections import Counter

from PIL import Image
from colormath.color_conversions import convert_color
from colormath.color_objects import sRGBColor, LabColor
from src.block_pb2 import Block, BlockStats, Lab, Weight


def _rgb_to_lab(r: int, g: int, b: int, step: float = 1.0) -> tuple[float, float, float]:
    """Convert a RGB value to a quantized Lab colour."""
    rgb = sRGBColor(r, g, b, is_upscaled=True)
    lab: LabColor = convert_color(rgb, LabColor)

    return round(lab.lab_l / step) * step, round(lab.lab_a / step) * step, round(lab.lab_b / step) * step


def _colour_frequency(image: Image.Image) -> list[tuple[tuple[float, float, float], int]]:
    """Returns list of ((l, a, b), count)"""
    # count in RGB
    rgba_freq = Counter(image.get_flattened_data())

    # convert unique colours only
    freq = Counter()

    for (r, g, b, _), count in rgba_freq.items():
        lab = _rgb_to_lab(r, g, b)
        freq[lab] += count

    return list(freq.items())


def _normalise(freq: list[tuple[tuple[float, float, float], int]]) -> list[Weight]:
    total = sum(count for _, count in freq)
    if total == 0:
        return []

    return [
        Weight(colour=Lab(l=l, a=a, b=b), weight=count / total)
        for (l, a, b), count in freq
    ]


def _average_lab(freq: list[tuple[tuple[float, float, float], int]]) -> Lab:
    total = sum(count for _, count in freq)
    if total == 0:
        return 0.0, 0.0, 0.0

    l = sum(l * c for (l, _, _), c in freq) / total
    a = sum(a * c for (_, a, _), c in freq) / total
    b = sum(b * c for (_, _, b), c in freq) / total

    return Lab(l=l, a=a, b=b)


def compute_stats(textures: dict[tuple[str, str | None], Image.Image]) -> list[BlockStats]:
    stats = []

    for (base, overlay), texture in textures.items():
        freq = _colour_frequency(texture)
        average = _average_lab(freq)
        weights = _normalise(freq)

        stats.append(BlockStats(block=Block(base=base, overlay=overlay), average=average, weights=weights))

    return stats
