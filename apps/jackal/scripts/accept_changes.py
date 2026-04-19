#!/usr/bin/env python3
import sys, zipfile, shutil, xml.etree.ElementTree as ET
from pathlib import Path
import tempfile

NS = {'w': 'http://schemas.openxmlformats.org/wordprocessingml/2006/main'}
ET.register_namespace('w', NS['w'])

def accept_all_changes(src, dst):
    tmp = Path(tempfile.mkdtemp())
    with zipfile.ZipFile(src, 'r') as z:
        z.extractall(tmp)

    doc_xml = tmp / "word" / "document.xml"
    tree = ET.parse(doc_xml)
    root = tree.getroot()

    parent_map = {c: p for p in root.iter() for c in p}

    # Удаляем удаления и перемещения (с защитой от вложенности)
    for tag in ('.//w:del', './/w:moveFrom', './/w:moveTo'):
        for elem in root.findall(tag, NS):
            parent = parent_map.get(elem)
            if parent is not None and elem in parent:
                parent.remove(elem)

    # Выносим содержимое вставок (с защитой от вложенности)
    for ins in root.findall('.//w:ins', NS):
        parent = parent_map.get(ins)
        if parent is not None and ins in parent:
            idx = list(parent).index(ins)
            for child in reversed(list(ins)):
                parent.insert(idx, child)
            parent.remove(ins)

    tree.write(doc_xml, encoding='utf-8', xml_declaration=True)

    with zipfile.ZipFile(dst, 'w', zipfile.ZIP_DEFLATED) as z:
        for f in tmp.rglob('*'):
            if f.is_file():
                z.write(f, f.relative_to(tmp))
    shutil.rmtree(tmp)

if __name__ == '__main__':
    if len(sys.argv) != 3:
        sys.exit("Usage: accept_changes.py input.docx output.docx")
    accept_all_changes(sys.argv[1], sys.argv[2])