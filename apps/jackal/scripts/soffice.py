#!/usr/bin/env python3
import sys, subprocess

if __name__ == '__main__':
    subprocess.run(["libreoffice", "--headless"] + sys.argv[1:], check=True)