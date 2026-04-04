import argparse

from requests import Session
from src.version import fetch_versions, resolve_versions


def get_http() -> Session:
    session = Session()
    return session


def get_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", "-v", help="The version of Minecraft to fetch data for")

    return parser.parse_args()


def main() -> None:
    # setup
    args = get_args()
    http = get_http()

    # fetch data
    versions = fetch_versions(http, args.version)
    versions = resolve_versions(http, versions)
    print(versions)

if __name__ == "__main__":
    main()
