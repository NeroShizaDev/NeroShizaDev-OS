#!/usr/bin/env python3
import sys, zipfile
from pathlib import Path

def pack(src, dst):
    src_path = Path(src)
    with zipfile.ZipFile(dst, 'w', zipfile.ZIP_DEFLATED) as z:
        for f in src_path.rglob('*'):
            if f.is_file():
                z.write(f, f.relative_to(src_path))

if __name__ == '__main__':
    if len(sys.argv) != 3:
        sys.exit("Usage: pack.py unpacked/ output.docx")
    pack(sys.argv[1], sys.argv[2])