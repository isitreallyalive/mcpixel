from collections import Counter

import numpy as np
from skimage import color as skcolor

from PIL import Image
from src.block_pb2 import Block, BlockStats, PlacedBlock, Weight, Lab
from src.proto import Hashable


def _rgb_to_lab(r: float, g: float, b: float) -> Lab:
    rgb = np.array([[[r / 255., g / 255., b / 255.]]])
    l, a, b_ = skcolor.rgb2lab(rgb)[0, 0]
    return Lab(l=float(l), a=float(a), b=float(b_))


def _colour_frequency(image: Image.Image) -> list[tuple[tuple[float, float, float], int]]:
    """Returns list of ((r, g, b), count)"""
    freq = Counter()
    for rgba in image.get_flattened_data():
        rgb = tuple(rgba[:3])
        freq[rgb] += 1
    return list(freq.items())


def _normalise(freq: list[tuple[tuple[float, float, float], int]]) -> list[Weight]:
    total = sum(count for _, count in freq)
    if total == 0:
        return []

    return [
        Weight(colour=_rgb_to_lab(r, g, b), weight=count / total)
        for (r, g, b), count in freq
    ]


def _average_lab(freq: list[tuple[tuple[float, float, float], int]]) -> Lab:
    total = sum(count for _, count in freq)
    if total == 0:
        return Lab(l=0., a=0., b=0.)

    # average in LAB space, not RGB
    rgb_array = np.array([[[r / 255., g / 255., b / 255.]] for (r, g, b), _ in freq])
    lab_array = skcolor.rgb2lab(rgb_array)[:, 0, :]
    weights = np.array([c for _, c in freq], dtype=np.float64)

    l, a, b = np.average(lab_array, axis=0, weights=weights)
    return Lab(l=float(l), a=float(a), b=float(b))


def compute_stats(textures: dict[tuple[Hashable[PlacedBlock], Hashable[PlacedBlock] | None], Image.Image]) -> list[
    BlockStats]:
    stats = []

    for (base, overlay), texture in textures.items():
        freq = _colour_frequency(texture)
        weights = _normalise(freq)
        average = _average_lab(freq)

        stats.append(BlockStats(
            block=Block(base=base.get_proto(), overlay=overlay.get_proto() if overlay else None),
            average=average,
            weights=weights,
        ))

    return stats
