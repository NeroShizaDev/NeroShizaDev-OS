#!/usr/bin/env python3
import sys, zipfile, shutil, xml.etree.ElementTree as ET
from pathlib import Path
import tempfile

NS = {'w': 'http://schemas.openxmlformats.org/wordprocessingml/2006/main'}
ET.register_namespace('w', NS['w'])

def add_comment(src, text, author="Claude"):
    tmp = Path(tempfile.mkdtemp())
    with zipfile.ZipFile(src, 'r') as z:
        z.extractall(tmp)

    doc_xml = tmp / "word" / "document.xml"
    tree = ET.parse(doc_xml)
    root = tree.getroot()
    body = root.find('.//w:body', NS)

    p = ET.Element(f"{{{NS['w']}}}p")
    r = ET.SubElement(p, f"{{{NS['w']}}}r")
    t = ET.SubElement(r, f"{{{NS['w']}}}t")
    t.text = f"[{author}]: {text}"

    # Ищем настройки секции (они должны быть последними в body)
    sect_pr = body.find('./w:sectPr', NS)
    if sect_pr is not None:
        idx = list(body).index(sect_pr)
        body.insert(idx, p)          # вставляем ПЕРЕД секцией
    else:
        body.append(p)

    tree.write(doc_xml, encoding='utf-8', xml_declaration=True)

    with zipfile.ZipFile(src, 'w', zipfile.ZIP_DEFLATED) as z:
        for f in tmp.rglob('*'):
            if f.is_file():
                z.write(f, f.relative_to(tmp))
    shutil.rmtree(tmp)

if __name__ == '__main__':
    if len(sys.argv) < 3:
        sys.exit("Usage: comment.py document.docx text [author]")
    author = sys.argv[3] if len(sys.argv) > 3 else "Claude"
    add_comment(sys.argv[1], sys.argv[2], author)