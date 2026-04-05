from collections import Counter

import numpy as np
from skimage import color as skcolor

from PIL import Image
from src.block_pb2 import Lab, PlacedBlock, Texture
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


def _normalise(freq: list[tuple[tuple[float, float, float], int]]) -> tuple[list[Lab], list[float]]:
    total = sum(count for _, count in freq)
    if total == 0:
        return [], []

    return [_rgb_to_lab(r, g, b) for (r, g, b), _ in freq], [count / total for _, count in freq]


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


def _smoothness(image: Image.Image) -> float:
    # ensure RGB
    img = image.convert("RGB")
    arr = np.asarray(img, dtype=np.float32) / 255.0  # (H, W, 3)

    # convert to Lab
    lab = skcolor.rgb2lab(arr)  # (H, W, 3)

    # horizontal differences
    dx = lab[:, 1:, :] - lab[:, :-1, :]
    # vertical differences
    dy = lab[1:, :, :] - lab[:-1, :, :]

    # squared distances
    dx2 = np.sum(dx * dx, axis=2)
    dy2 = np.sum(dy * dy, axis=2)

    total = dx2.sum() + dy2.sum()
    count = dx2.size + dy2.size

    if count == 0:
        return 0.0

    return float(total / count)


def compute_stats(textures: dict[tuple[Hashable[PlacedBlock], Hashable[PlacedBlock] | None], Image.Image]) -> list[
    Texture]:
    stats = []

    for (base, overlay), texture in textures.items():
        freq = _colour_frequency(texture)
        colours, weights = _normalise(freq)
        average = _average_lab(freq)
        smoothness = _smoothness(texture)

        stats.append(Texture(
            base=base.get_proto(),
            overlay=overlay.get_proto() if overlay else None,
            average=average,
            smoothness=smoothness,
            colours=colours,
            weights=weights,
        ))

    return stats
