#!/usr/bin/env python3
import sys, subprocess
from pathlib import Path

def extract_text(docx_path, output_md=None):
    if output_md is None:
        output_md = Path(docx_path).with_suffix('.md')
    try:
        subprocess.run(['pandoc', docx_path, '-o', output_md, '--track-changes=all'],
            check=True
        )
        print(f"Text extracted to {output_md}")
    except FileNotFoundError:
        sys.exit("pandoc not found. Install pandoc first.")
    except subprocess.CalledProcessError as e:
        sys.exit(f"pandoc failed: {e}")

if __name__ == '__main__':
    if len(sys.argv) < 2:
        sys.exit("Usage: extract_text.py document.docx [output.md]")
    out = sys.argv[2] if len(sys.argv) > 2 else None
    extract_text(sys.argv[1], out)