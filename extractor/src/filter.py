import json
import subprocess
from pathlib import Path
from tempfile import TemporaryDirectory

from PIL import Image
from src.version import Version

SHAPE_PROPS = {"half", "type", "facing", "layers", "hinge", "shape", "part"}


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
    )

    if block_id in non_solid:
        return False

    # don't allow shape-variant blocks
    props = (entry or {}).get("properties", {})
    if SHAPE_PROPS & props.keys():
        return False

    return True


def filter_textures(version: Version, textures: dict[str, Image.Image], tags: dict[str, set[str]]) -> dict[str, Image.Image]:
    """Remove textures that should not be considered."""
    report = _generate_report(version)

    return {block_id: img for block_id, img in textures.items() if _is_valid_texture(block_id, report, tags)}
