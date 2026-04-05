from PIL import Image
from src.block_pb2 import PlacedBlock
from src.proto import Hashable


def apply_overlays(textures: dict[Hashable[PlacedBlock], Image.Image]) -> dict[
    tuple[Hashable[PlacedBlock], Hashable[PlacedBlock] | None], Image.Image]:
    """Apply overlays to a textures dict"""
    output = {}

    # find overlays
    overlays = {
        block: img
        for block, img in textures.items()
        if block.id.endswith("glass")
    }

    # find bases
    bases = {
        block: img
        for block, img in textures.items()
        if block not in overlays
    }

    for base, base_img in bases.items():
        # add the texture alone
        output[(base, None)] = base_img

        # add all possible overlays
        for overlay, overlay_img in overlays.items():
            # apply the overlay
            result = base_img.copy()
            overlay_img = overlay_img.resize(base_img.size, Image.Resampling.NEAREST)
            result.paste(overlay_img, mask=overlay_img.split()[3])

            # add it to the output
            output[(base, overlay)] = result

    return output
