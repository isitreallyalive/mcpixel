import argparse
import logging

from requests import Session
from src.stats import compute_stats
from src.texture import extract_textures, filter_textures, apply_overlays
from src.version import fetch_versions, resolve_versions


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

    return parser.parse_args()


def main() -> None:
    # setup
    logger = get_logger()
    args = get_args()
    http = get_http()

    # fetch data
    versions = fetch_versions(http, args.version)
    versions = resolve_versions(http, versions)

    for version in versions.values():
        textures = extract_textures(version)
        logger.info(f"Extracted {len(textures):,} textures for version {version}")

        textures = filter_textures(textures)
        logger.info(f"Filtered to {len(textures):,} textures")

        textures = apply_overlays(textures)
        logger.info(f"Overlays applied, {len(textures):,} possible combinations")

        stats = compute_stats(textures)
        logger.info(f"Computed stats for {len(stats):,} textures")

if __name__ == "__main__":
    main()
