from dataclasses import dataclass
from io import BytesIO
from pathlib import Path
from typing import Literal
from zipfile import ZipFile

import orjson
from PIL import Image
from requests import Session
from src.block_pb2 import PlacedBlock
from src.path import DOWNLOAD_DIR
from src.proto import Hashable
from tqdm import tqdm

Side = Literal["server", "client"]

MANIFEST_URL = "https://piston-meta.mojang.com/mc/game/version_manifest.json"
CHUNK_SIZE = 1024


@dataclass
class Version:
    name: str
    client: Path
    server: Path | None = None


def fetch_versions(http: Session, version: str | None = None) -> dict[str, str]:
    """Returns a dictionary mapping versions of Minecraft to a URL to their corresponding client.json"""
    # fetch manifest
    with http.get(MANIFEST_URL) as res:
        res.raise_for_status()
        manifest = res.json()

    # filter to releases only
    versions = manifest.get("versions", [])
    versions = filter(lambda v: v["type"] == "release", versions)

    # map to version: url
    versions = {v.get("id"): v.get("url") for v in versions}
    if version is not None:
        if version not in versions:
            raise ValueError(f"Version '{version}' does not exist!")
        return {version: versions[version]} if version in versions else {}

    return versions


def _fetch_jar(http: Session, side: Side, version: str, downloads: dict[str, dict[str, str]]) -> Path | None:
    """Fetch a Minecraft version's jar."""
    path = DOWNLOAD_DIR / f"{version}.{side}.jar"

    # if it doesn't already exist, download it
    if not path.exists():
        download_url = downloads.get(side, {}).get("url")

        if download_url is None:
            return None

        # download the jar
        with http.get(download_url, stream=True) as res:
            res.raise_for_status()
            total = int(res.headers.get("content-length", 0))

            with open(path, "wb") as f, tqdm(total=total, unit="B", unit_scale=True, desc=f"{version}-{side}") as bar:
                for chunk in res.iter_content(chunk_size=CHUNK_SIZE):
                    if chunk:
                        f.write(chunk)
                        bar.update(len(chunk))

    return path


def resolve_versions(http: Session, versions: dict[str, str]) -> list[Version]:
    """Resolves versions to their client and server jars."""
    resolved = []

    for version, url in versions.items():
        # find the version's downloads
        with http.get(url) as res:
            res.raise_for_status()
            client_json = res.json()

        downloads = client_json.get("downloads", {})

        # fetch the relevant jars
        client = _fetch_jar(http, "client", version, downloads)
        server = _fetch_jar(http, "server", version, downloads)

        if client is not None:
            resolved.append(Version(version, client, server))

    return resolved


def extract_version(version: Version) -> tuple[dict[Hashable[PlacedBlock], Image.Image], dict[str, set[str]]]:
    """Extract textures and tag membership from a client jar."""
    textures = {}
    tags = {}

    with (ZipFile(version.client, "r") as jar):
        for path in jar.namelist():
            name = Path(path).stem

            # handle textures
            if (
                    path.startswith("assets/minecraft/textures/block/")  # 1.13+
                    or path.startswith("assets/minecraft/textures/blocks/") # 1.6+
                or path.startswith("textures/blocks/")
            ) and path.endswith(".png"):
                # read the texture
                data = jar.read(path)
                image = Image.open(BytesIO(data)).convert("RGBA")

                # animated textures are tall strips - only use the first frame
                w, h = image.size
                if h > w:
                    image = image.crop((0, 0, w, h))

                # is it a _top texture?
                if name.endswith("top"):
                    name = name.strip("_top")
                    top = True
                else:
                    top = False

                textures[Hashable(PlacedBlock(id=name, top=top))] = image

            # handle tags

            if (
                    path.startswith("data/minecraft/tags/block/")  # 1.21+
                    or path.startswith("data/minecraft/tags/blocks/")
            ) and path.endswith(".json"):
                data = orjson.loads(jar.read(path))
                tags[name] = {
                    v.lstrip("#").split(":")[1]
                    for v in data.get("values", [])
                    if isinstance(v, str)
                }

    return textures, tags
