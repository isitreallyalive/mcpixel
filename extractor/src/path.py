from pathlib import Path

EXTRACTOR_DIR = Path(__file__).resolve().parent.parent
BASE_DIR = EXTRACTOR_DIR.parent
DATA_DIR = BASE_DIR / "data"
DOWNLOAD_DIR = EXTRACTOR_DIR / ".downloads"

# make sure important directories exist
DATA_DIR.mkdir(parents=True, exist_ok=True)
DOWNLOAD_DIR.mkdir(parents=True, exist_ok=True)