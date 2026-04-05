# todo: support pre-1.5 terrain.png

import argparse
import logging

from google.protobuf import json_format
from requests import Session
from src.block_pb2 import Version
from src.filter import filter_textures
from src.path import DATA_DIR
from src.stats import compute_stats
from src.texture import apply_overlays
from src.version import fetch_versions, resolve_versions, extract_version



def get_logger() -> logging.Logger:
    logging.basicConfig(level=logging.INFO, format="[%(levelname)s] %(message)s")
    logger = logging.getLogger(__name__)

    return logger


def get_http() -> Session:
    session = Session()

    return session


def get_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", "-v", help="The version of Minecraft to fetch data for")
    parser.add_argument("--overwrite", "-o", help="Overwrite existing files")

    return parser.parse_args()


def main() -> None:
    # setup
    logger = get_logger()
    args = get_args()
    http = get_http()

    # fetch data
    versions = fetch_versions(http, args.version)
    versions = resolve_versions(http, versions)

    for version in versions:
        out_path = DATA_DIR / version.name
        if not args.overwrite and out_path.exists():
            continue

        textures, tags = extract_version(version)
        logger.info(f"Extracted {len(textures):,} textures and {len(tags):,} tags for version {version.name}")

        textures = filter_textures(version, textures, tags)
        logger.info(f"Filtered to {len(textures):,} textures")

        textures = apply_overlays(textures)
        logger.info(f"Overlays applied, {len(textures):,} possible combinations")

        stats = compute_stats(textures)
        logger.info(f"Computed stats for {len(stats):,} textures")

        output = Version(stats=stats)

        with open(out_path, "wb") as f:
            f.write(output.SerializeToString())
        with open(DATA_DIR / f"{version.name}.json", "w") as f:
            f.write(json_format.MessageToJson(output, indent=1, always_print_fields_with_no_presence=True))


if __name__ == "__main__":
    main()
