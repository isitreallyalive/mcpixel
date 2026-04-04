from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from requests import Session
from src.path import DOWNLOAD_DIR
from tqdm import tqdm

Side = Literal["server", "client"]

MANIFEST_URL = "https://piston-meta.mojang.com/mc/game/version_manifest.json"
CHUNK_SIZE = 1024


@dataclass
class Version:
    name: str
    client: Path
    server: Path

    def __str__(self):
        return self.name


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

def _fetch_jar(http: Session, side: Side, version: str, downloads: dict[str, dict[str, str]]) -> Path:
    """Fetch a Minecraft version's jar."""
    path = DOWNLOAD_DIR / f"{version}.{side}.jar"

    # if it doesn't already exist, download it
    if not path.exists():
        download_url = downloads.get(side, {}).get("url")

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


def resolve_versions(http: Session, versions: dict[str, str]) -> dict[str, Version]:
    """Resolves versions to their client and server jars."""
    resolved = {}

    for version, url in versions.items():
        # find the version's downloads
        with http.get(url) as res:
            res.raise_for_status()
            client_json = res.json()

        downloads = client_json.get("downloads", {})

        # fetch the relevant jars
        client = _fetch_jar(http, "client", version, downloads)
        server = _fetch_jar(http, "server", version, downloads)

        resolved[version] = Version(version, client, server)

    return resolved
