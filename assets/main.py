import json
import subprocess
from argparse import ArgumentParser
from collections import Counter
from dataclasses import dataclass, asdict
from io import BytesIO
from pathlib import Path
from tempfile import TemporaryDirectory
from zipfile import ZipFile

import msgpack
from PIL import Image

BASE_DIR = Path(__file__).resolve().parent

# todo: check block tags
BAD_PROPS = {"half", "type", "facing", "layers", "attachment",
             "open", "in_wall", "extended", "hinge"}

BAD_SUFFIXES = (
    "_slab", "_stairs", "_wall", "_fence", "_gate",
    "_door", "_trapdoor", "_button", "_pressure_plate",
    "_sign", "_banner", "_bed", "_carpet", "_torch",
    "_rail", "_pane", "_rod", "_chain", "_sapling", "_propagule",
    "_flower", "_mushroom", "_coral", "_kelp", "_vine",
    "_shoot", "_head", "_skull", "_candle", "_glass"
)

BAD_EXACT = {
    "air", "cave_air", "void_air", "water", "lava", "fire",
    "soul_fire", "cobweb", "scaffolding", "ladder", "snow",
    "grass", "fern", "dead_bush", "seagrass", "bubble_column",
    "flower_pot"
}


@dataclass
class Combo:
    b: str  # base
    a: list[float]  # average rgb
    c: list[list[int]]  # colour frequency
    o: str | None = None  # overlay


def is_valid_base(block_id: str, report: dict) -> bool:
    full_id = f"minecraft:{block_id}"
    entry = report.get(full_id)
    if entry is None:
        return False
    if block_id in BAD_EXACT:
        return False
    if any(block_id.endswith(s) for s in BAD_SUFFIXES):
        return False
    props = entry.get("properties", {})
    if BAD_PROPS & props.keys():
        return False
    return True


def is_valid_overlay(block_id: str) -> bool:
    return block_id.endswith("_stained_glass")


def extract_textures(path: Path) -> dict[str, Image.Image]:
    """Extracts all block textures."""
    textures = {}
    with ZipFile(path, "r") as jar:
        for name in jar.namelist():
            if not name.startswith("assets/minecraft/textures/block/"):
                continue
            if not name.endswith(".png"):
                continue
            block_id = Path(name).stem
            data = jar.read(name)
            img = Image.open(BytesIO(data)).convert("RGBA")
            # animated textures are tall strips - only use the first frame
            w, h = img.size
            if h > w:
                img = img.crop((0, 0, w, w))
            textures[block_id] = img
    return textures


def generate_report(server_jar: Path) -> dict:
    """Generates the blocks.json report using the server jar."""
    with TemporaryDirectory() as tmp:
        subprocess.run(
            ["java", "-DbundlerMainClass=net.minecraft.data.Main",
             "-jar", str(server_jar.resolve()), "--reports"],
            cwd=tmp,
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        report_path = Path(tmp) / "generated" / "reports" / "blocks.json"
        return json.loads(report_path.read_bytes())


def color_frequency(img: Image.Image) -> list[tuple[int, int, int, int, int]]:
    """Returns [(r, g, b, a, count), ...] for all sufficiently opaque pixels."""
    counts = Counter()
    for r, g, b, a in img.get_flattened_data():
        if a < 128:
            continue
        counts[(r, g, b, a)] += 1
    return [(r, g, b, a, c) for (r, g, b, a), c in counts.most_common()]


def average_rgb(freq: list[tuple[int, int, int, int, int]]) -> tuple[float, float, float]:
    total = sum(c for _, _, _, _, c in freq)
    if total == 0:
        return 0., 0., 0.
    r = sum(r * c for r, _, _, _, c in freq) / total
    g = sum(g * c for _, g, _, _, c in freq) / total
    b = sum(b * c for _, _, b, _, c in freq) / total
    a = sum(a * c for _, _, _, a, c in freq) / total
    return r, g, b, a


def composite(base: Image.Image, overlay: Image.Image) -> Image.Image:
    overlay = overlay.resize(base.size, Image.Resampling.NEAREST)
    result = base.copy()
    result.paste(overlay, mask=overlay.split()[3])
    return result


def make_combo(base_id: str, img: Image.Image, overlay_id: str | None = None) -> Combo | None:
    freq = color_frequency(img)
    if not freq:
        return None
    avg = average_rgb(freq)
    return Combo(base_id, list(avg), [[r, g, b, a, c] for r, g, b, a, c in freq], overlay_id)


def build_combos(textures: dict[str, Image.Image], report: dict) -> list[Combo]:
    base_ids = sorted(bid for bid in textures if is_valid_base(bid, report))
    overlay_ids = sorted(bid for bid in textures if is_valid_overlay(bid))

    print(f"Found {len(base_ids)} bases and {len(overlay_ids)} overlays")

    combos = []
    for base_id in base_ids:
        base_img = textures[base_id]

        # single layer
        combo = make_combo(base_id, base_img)
        if combo:
            combos.append(combo)

        # double layer
        for overlay_id in overlay_ids:
            comp = composite(base_img, textures[overlay_id])
            combo = make_combo(base_id, comp, overlay_id)
            if combo:
                combos.append(combo)

    return combos


def write_combos(combos: list[Combo], version: str) -> None:
    out_path = BASE_DIR / f"{version}.msgpack"
    with open(out_path, "wb") as f:
        msgpack.pack([asdict(c) for c in combos], f)
    print(f"Wrote {len(combos)} combos to {out_path}")


def process(client_path: Path, server_path: Path, version: str) -> None:
    report = generate_report(server_path)
    textures = extract_textures(client_path)
    combos = build_combos(textures, report)
    print(f"Generated {len(combos)} combos")
    write_combos(combos, version)


def main() -> None:
    parser = ArgumentParser()
    parser.add_argument("client", type=Path, help="client jar")
    parser.add_argument("server", type=Path, help="server jar")
    parser.add_argument("version")
    args = parser.parse_args()
    process(args.client, args.server, args.version)


if __name__ == "__main__":
    main()
