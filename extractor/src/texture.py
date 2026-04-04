from dataclasses import dataclass
from io import BytesIO
from pathlib import Path
from zipfile import ZipFile

from PIL import Image
from src.version import Version

@dataclass(frozen=True)
class Block:
    base: str
    overlay: str | None = None

def extract_textures(version: Version) -> dict[str, Image.Image]:
    """Extract textures from a version's client.jar"""
    textures = {}

    with ZipFile(version.client, "r") as jar:
        for path in jar.namelist():
            # only consider blocks
            if not path.startswith("assets/minecraft/textures/block/"):
                continue
            # which are PNGs
            if not path.endswith(".png"):
                continue

            # read the texture
            block = Path(path).stem
            data = jar.read(path)
            image = Image.open(BytesIO(data)).convert("RGBA")

            # animated textures are tall strips - only use the first frame
            w, h = image.size
            if h > w:
                image = image.crop((0, 0, w, h))

            textures[block] = image

    return textures

def filter_textures(textures: dict[str, Image.Image]) -> dict[str, Image.Image]:
    """Remove textures that should not be considered."""
    # todo: filter out bad textures
    return textures


def apply_overlays(textures: dict[str, Image.Image]) -> dict[Block, Image.Image]:
    """Apply overlays to a textures dict"""
    output = {}

    # find overlays
    overlays = {
        name: img
        for name, img in textures.items()
        if name.endswith("_stained_glass")
    }

    for base_name, base in textures.items():
        # add the texture alone
        output[Block(base_name)] = base

        # add all possible overlays
        for overlay_name, overlay in overlays.items():
            # apply the overlay
            result = base.copy()
            overlay = overlay.resize(base.size, Image.Resampling.NEAREST)
            result.paste(overlay, mask=overlay.split()[3])

            # add it to the output
            output[Block(base_name, overlay_name)] = base_name

    return output

