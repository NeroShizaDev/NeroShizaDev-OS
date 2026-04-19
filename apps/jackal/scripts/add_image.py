#!/usr/bin/env python3
import sys, zipfile, shutil, xml.etree.ElementTree as ET
from pathlib import Path
import tempfile
import uuid

NS = {
    'w': 'http://schemas.openxmlformats.org/wordprocessingml/2006/main',
    'r': 'http://schemas.openxmlformats.org/package/2006/relationships',
    'pic': 'http://schemas.openxmlformats.org/drawingml/2006/picture',
    'a': 'http://schemas.openxmlformats.org/drawingml/2006/main',
    'wp': 'http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing',
}
for prefix, uri in NS.items():
    ET.register_namespace(prefix, uri)

def add_image(docx_path, image_path, caption=None):
    tmp = Path(tempfile.mkdtemp())
    with zipfile.ZipFile(docx_path, 'r') as z:
        z.extractall(tmp)

    media_dir = tmp / "word" / "media"
    media_dir.mkdir(exist_ok=True)
    img_name = Path(image_path).name
    dest_img = media_dir / img_name
    shutil.copy2(image_path, dest_img)

    ext = Path(img_name).suffix.lower()
    mime = {
        '.png': 'image/png',
        '.jpg': 'image/jpeg',
        '.jpeg': 'image/jpeg',
        '.gif': 'image/gif',
        '.bmp': 'image/bmp'
    }.get(ext, 'image/png')

    types_path = tmp / "[Content_Types].xml"
    types_tree = ET.parse(types_path)
    types_root = types_tree.getroot()
    found = False
    for default in types_root.findall('{http://schemas.openxmlformats.org/package/2006/content-types}Default'):
        if default.get('Extension') == ext[1:]:
            found = True
            break
    if not found:
        ET.SubElement(types_root, '{http://schemas.openxmlformats.org/package/2006/content-types}Default',
                      {'Extension': ext[1:], 'ContentType': mime})
    types_tree.write(types_path, encoding='utf-8', xml_declaration=True)

    rels_path = tmp / "word" / "_rels" / "document.xml.rels"
    rels_tree = ET.parse(rels_path)
    rels_root = rels_tree.getroot()

    # Ищем максимальный rId, чтобы избежать коллизий
    existing_ids = [rel.get('Id') for rel in rels_root.findall('{http://schemas.openxmlformats.org/package/2006/relationships}Relationship')]
    max_num = 0
    for rid in existing_ids:
        if rid.startswith('rId') and rid[3:].isdigit():
            max_num = max(max_num, int(rid[3:]))
    rel_id = f"rId{max_num + 1}"

    ET.SubElement(rels_root, '{http://schemas.openxmlformats.org/package/2006/relationships}Relationship', {
        'Id': rel_id,
        'Type': 'http://schemas.openxmlformats.org/officeDocument/2006/relationships/image',
        'Target': f'media/{img_name}'
    })
    rels_tree.write(rels_path, encoding='utf-8', xml_declaration=True)

    doc_path = tmp / "word" / "document.xml"
    doc_tree = ET.parse(doc_path)
    doc_root = doc_tree.getroot()
    body = doc_root.find('.//w:body', NS)

    p = ET.Element(f"{{{NS['w']}}}p")
    r = ET.SubElement(p, f"{{{NS['w']}}}r")
    drawing = ET.SubElement(r, f"{{{NS['w']}}}drawing")
    inline = ET.SubElement(drawing, f"{{{NS['wp']}}}inline")
    ET.SubElement(inline, f"{{{NS['wp']}}}extent", {'cx': '914400', 'cy': '914400'})
    ET.SubElement(inline, f"{{{NS['wp']}}}effectExtent", {'l': '0', 't': '0', 'r': '0', 'b': '0'})
    docPr = ET.SubElement(inline, f"{{{NS['wp']}}}docPr", {'id': str(uuid.uuid4().int % 1000000), 'name': img_name})

    ET.SubElement(inline, f"{{{NS['wp']}}}cNvGraphicFramePr")  # исправлены кавычки

    graphic = ET.SubElement(inline, f"{{{NS['a']}}}graphic")
    graphicData = ET.SubElement(graphic, f"{{{NS['a']}}}graphicData", {'uri': 'http://schemas.openxmlformats.org/drawingml/2006/picture'})
    pic = ET.SubElement(graphicData, f"{{{NS['pic']}}}pic")
    nvPicPr = ET.SubElement(pic, f"{{{NS['pic']}}}nvPicPr")
    cNvPr = ET.SubElement(nvPicPr, f"{{{NS['pic']}}}cNvPr", {'id': '0', 'name': img_name})
    cNvPicPr = ET.SubElement(nvPicPr, f"{{{NS['pic']}}}cNvPicPr")
    blipFill = ET.SubElement(pic, f"{{{NS['pic']}}}blipFill")
    blip = ET.SubElement(blipFill, f"{{{NS['a']}}}blip", {f"{{{NS['r']}}}embed": rel_id})
    stretch = ET.SubElement(blipFill, f"{{{NS['a']}}}stretch")
    ET.SubElement(stretch, f"{{{NS['a']}}}fillRect")
    spPr = ET.SubElement(pic, f"{{{NS['pic']}}}spPr")
    xfrm = ET.SubElement(spPr, f"{{{NS['a']}}}xfrm")
    ET.SubElement(xfrm, f"{{{NS['a']}}}off", {'x': '0', 'y': '0'})
    ET.SubElement(xfrm, f"{{{NS['a']}}}ext", {'cx': '914400', 'cy': '914400'})

    prstGeom = ET.SubElement(spPr, f"{{{NS['a']}}}prstGeom", {'prst': 'rect'})
    ET.SubElement(prstGeom, f"{{{NS['a']}}}avLst")  # исправлены кавычки

    body.append(p)

    if caption:
        cap_p = ET.Element(f"{{{NS['w']}}}p")
        cap_r = ET.SubElement(cap_p, f"{{{NS['w']}}}r")
        cap_t = ET.SubElement(cap_r, f"{{{NS['w']}}}t")
        cap_t.text = caption
        body.append(cap_p)

    doc_tree.write(doc_path, encoding='utf-8', xml_declaration=True)

    with zipfile.ZipFile(docx_path, 'w', zipfile.ZIP_DEFLATED) as z:
        for f in tmp.rglob('*'):
            if f.is_file():
                z.write(f, f.relative_to(tmp))
    shutil.rmtree(tmp)

if __name__ == '__main__':
    if len(sys.argv) < 3:
        sys.exit("Usage: add_image.py document.docx image.jpg [caption]")
    cap = sys.argv[3] if len(sys.argv) > 3 else None
    add_image(sys.argv[1], sys.argv[2], cap)