#!/usr/bin/env python3
import sys, zipfile
from pathlib import Path

def unpack(src, dst):
    with zipfile.ZipFile(src, 'r') as z:
        z.extractall(dst)

if __name__ == '__main__':
    if len(sys.argv) != 3:
        sys.exit("Usage: unpack.py doc.docx unpacked/")
    unpack(sys.argv[1], sys.argv[2])