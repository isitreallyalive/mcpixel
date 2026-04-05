import json
import re
import subprocess
from pathlib import Path
from tempfile import TemporaryDirectory

import semver

from PIL import Image
from src.block_pb2 import PlacedBlock
from src.proto import Hashable
from src.version import Version

NO_REPORT = semver.Version.parse("1.17.1")

# todo: refine blacklist
SHAPE_PROPS = {"half", "type", "facing", "layers", "hinge", "shape", "part"}
BAD_ITEMS = {"dragon_egg", "mycelium", "conduit", "flower_pot", "frogspawn", "test_instance_block", "daylight_detector",
             "cake", "enchanting_table", "scaffolding"}
BAD_SUFFIXES = {"glass_pane", "carpet", "vines", "torch", "lantern", "_grate"}


def _generate_report(version: Version) -> dict | None:
    with TemporaryDirectory() as tmp:
        subprocess.run(
            ["java", "-DbundlerMainClass=net.minecraft.data.Main", "-jar", str(version.server.resolve()), "--reports"],
            cwd=tmp, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        report_path = Path(tmp) / "generated" / "reports" / "blocks.json"
        if not report_path.exists():
            return None
        return json.loads(report_path.read_bytes())


def _is_valid_texture(block_id: str, report: dict | None, tags: dict[str, set[str]]) -> bool:
    # report doesn't exist pre-1.18
    # can anything be done to refine the data for these versions?
    if report is not None:
        # make sure the block exists
        full_id = f"minecraft:{block_id}"
        entry = report.get(full_id)
        if entry is None:
            return False

        # don't allow shape-variant blocks
        props = (entry or {}).get("properties", {})
        if SHAPE_PROPS & props.keys():
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
    return True


def _parse_version(v: str) -> semver.Version:
    # strip prerelease/build metadata (e.g. -pre1, -rc1)
    v = re.split(r"[-+]", v)[0]

    parts = v.split(".")
    parts += ["0"] * (3 - len(parts))  # pad to 3 components

    return semver.Version.parse(".".join(parts[:3]))


def filter_textures(version: Version, textures: dict[Hashable[PlacedBlock], Image.Image], tags: dict[str, set[str]]) -> \
        dict[Hashable[PlacedBlock], Image.Image]:
    """Remove textures that should not be considered."""
    report = _generate_report(version) if _parse_version(version.name) > NO_REPORT else None

    return {block: img for block, img in textures.items() if _is_valid_texture(block.id, report, tags)}
