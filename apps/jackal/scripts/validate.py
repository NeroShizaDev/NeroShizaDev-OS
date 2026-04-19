#!/usr/bin/env python3
import sys, zipfile

if __name__ == '__main__':
    if len(sys.argv) != 2:
        sys.exit("Usage: validate.py file.docx")
    try:
        with zipfile.ZipFile(sys.argv[1], 'r') as z:
            if 'word/document.xml' not in z.namelist():
                sys.exit("Invalid DOCX: missing word/document.xml")
        print("Valid DOCX")
    except Exception as e:
        sys.exit(f"Invalid: {e}")