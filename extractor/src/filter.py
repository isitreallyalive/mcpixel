import json
import subprocess
from pathlib import Path
from tempfile import TemporaryDirectory

from PIL import Image
from src.block_pb2 import PlacedBlock
from src.proto import Hashable
from src.version import Version

# todo: refine blacklist
SHAPE_PROPS = {"half", "type", "facing", "layers", "hinge", "shape", "part"}
BAD_ITEMS = {"dragon_egg", "mycelium", "conduit", "flower_pot", "frogspawn", "test_instance_block", "daylight_detector",
             "cake", "enchanting_table", "scaffolding"}
BAD_SUFFIXES = {"glass_pane", "carpet", "vines", "torch", "lantern", "_grate"}


def _generate_report(version: Version) -> dict:
    with TemporaryDirectory() as tmp:
        subprocess.run(
            ["java", "-DbundlerMainClass=net.minecraft.data.Main", "-jar", str(version.server.resolve()), "--reports"],
            cwd=tmp, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        report_path = Path(tmp) / "generated" / "reports" / "blocks.json"
        return json.loads(report_path.read_bytes())


def _is_valid_texture(block_id: str, report: dict, tags: dict[str, set[str]]) -> bool:
    # make sure the block exists
    full_id = f"minecraft:{block_id}"
    entry = report.get(full_id)
    if entry is None:
        return False

    # don't allow blacklisted items
    if block_id in BAD_ITEMS:
        return False
    for suffix in BAD_SUFFIXES:
        if block_id.endswith(suffix):
            return False

    # don't allow non-solid blocks
    non_solid = (
            tags.get("replaceable", set())
            | tags.get("fire", set())
            | tags.get("flowers", set())
            | tags.get("doors", set())
            | tags.get("trapdoors", set())
            | tags.get("fences", set())
            | tags.get("fence_gates", set())
            | tags.get("signs", set())
            | tags.get("banners", set())
            | tags.get("beds", set())
            | tags.get("buttons", set())
            | tags.get("pressure_plates", set())
            | tags.get("rails", set())
            | tags.get("stairs", set())
            | tags.get("slabs", set())
            | tags.get("walls", set())
            | tags.get("candles", set())
            | tags.get("carpets", set())
            | tags.get("saplings", set())
            | tags.get("coral_plants", set())
            | tags.get("small_flowers", set())
            | tags.get("tall_flowers", set())
            | tags.get("standing_signs", set())
            | tags.get("wall_signs", set())
            | tags.get("chains", set())
            | tags.get("bars", set())
            | tags.get("leaves", set())
            | tags.get("corals", set())
    )

    if block_id in non_solid:
        return False

    # don't allow shape-variant blocks
    props = (entry or {}).get("properties", {})
    if SHAPE_PROPS & props.keys():
        return False

    if "flower" in block_id:
        print(block_id)

    return True


def filter_textures(version: Version, textures: dict[Hashable[PlacedBlock], Image.Image], tags: dict[str, set[str]]) -> \
        dict[Hashable[PlacedBlock], Image.Image]:
    """Remove textures that should not be considered."""
    report = _generate_report(version)

    return {block: img for block, img in textures.items() if _is_valid_texture(block.id, report, tags)}
