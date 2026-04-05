from collections import Counter

from PIL import Image
from src.block_pb2 import Block, BlockStats, PlacedBlock, Weight, Rgb
from src.proto import Hashable


def _colour_frequency(image: Image.Image) -> list[tuple[tuple[float, float, float], int]]:
    """Returns list of ((r, g, b), count)"""
    freq = Counter()
    for rgba in image.get_flattened_data():
        rgb = tuple(rgba[:3])  # drop alpha
        freq[rgb] += 1
    return list(freq.items())


def _normalise(freq: list[tuple[tuple[float, float, float], int]]) -> list[Weight]:
    total = sum(count for _, count in freq)
    if total == 0:
        return []

    return [
        Weight(colour=Rgb(r=r, g=g, b=b), weight=count / total)
        for (r, g, b), count in freq
    ]


def _average_rgb(freq: list[tuple[tuple[float, float, float], int]]) -> Rgb:
    total = sum(count for _, count in freq)
    if total == 0:
        return 0.0, 0.0, 0.0

    r = sum(r * c for (r, _, _), c in freq) / total
    g = sum(g * c for (_, g, _), c in freq) / total
    b = sum(b * c for (_, _, b), c in freq) / total

    return Rgb(r=r, g=g, b=b)


def compute_stats(textures: dict[tuple[Hashable[PlacedBlock], Hashable[PlacedBlock] | None], Image.Image]) -> list[
    BlockStats]:
    stats = []

    for (base, overlay), texture in textures.items():
        freq = _colour_frequency(texture)
        average = _average_rgb(freq)
        weights = _normalise(freq)

        stats.append(BlockStats(block=Block(base=base.get_proto(), overlay=overlay.get_proto() if overlay else None),
                                average=average, weights=weights))

    return stats
