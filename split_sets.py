#!/usr/bin/env python3
"""Split the WWG card gallery CSV into one CSV file per Set."""

import argparse
import csv
import re
from pathlib import Path


def sanitize_filename(name: str) -> str:
    name = name.replace("\n", " ")
    name = re.sub(r"[:/\\?*\"<>|]", "-", name)
    name = re.sub(r"\s+", " ", name).strip()
    return name


def split_by_set(input_path: Path, output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)

    with input_path.open(newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        fieldnames = reader.fieldnames
        rows_by_set: dict[str, list[dict]] = {}
        for row in reader:
            rows_by_set.setdefault(row["Set"], []).append(row)

    for set_name, rows in rows_by_set.items():
        filename = sanitize_filename(set_name) + ".csv"
        out_path = output_dir / filename
        with out_path.open("w", newline="", encoding="utf-8") as f:
            writer = csv.DictWriter(f, fieldnames=fieldnames) # type: ignore
            writer.writeheader()
            writer.writerows(rows)
        print(f"{filename}: {len(rows)} card(s)")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "input",
        nargs="?",
        default="WWG Card Galleries - Public - Star Realms.csv",
        type=Path,
    )
    parser.add_argument("-o", "--output-dir", default="sets", type=Path)
    args = parser.parse_args()

    split_by_set(args.input, args.output_dir)


if __name__ == "__main__":
    main()
