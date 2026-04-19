#!/usr/bin/env python3
import sys, json, copy, zipfile, shutil, argparse, xml.etree.ElementTree as ET
from pathlib import Path
import tempfile

NS = {'w': 'http://schemas.openxmlformats.org/wordprocessingml/2006/main'}
ET.register_namespace('w', NS['w'])

def fill_table(template, output, data_json, skip_header=False, table_index=0):
    with open(data_json, 'r', encoding='utf-8') as f:
        data = json.load(f)

    tmp = Path(tempfile.mkdtemp())
    with zipfile.ZipFile(template, 'r') as z:
        z.extractall(tmp)

    doc_xml = tmp / "word" / "document.xml"
    tree = ET.parse(doc_xml)
    root = tree.getroot()
    
    tables = root.findall('.//w:tbl', NS)
    if table_index >= len(tables):
        sys.exit(f"Table index {table_index} not found.")
    table = tables[table_index]
    
    rows = table.findall('.//w:tr', NS)
    if not rows:
        sys.exit("No rows in table")
    
    t_row = rows[1] if skip_header and len(rows) > 1 else rows[0]

    # Удаляем все строки, начиная со строки-шаблона
    for r in rows[rows.index(t_row):]:
        table.remove(r)

    for row_data in data:
        nr = copy.deepcopy(t_row)
        cells = nr.findall('./w:tc', NS)
        col_idx = 0
        for cell in cells:
            # Пропускаем объединённые ячейки
            gs = cell.find('./w:tcPr/w:gridSpan', NS)
            if gs is not None and gs.get(f"{{{NS['w']}}}val") != '1':
                continue
            if col_idx < len(row_data):
                p = cell.find('./w:p', NS)
                if p is not None:
                    runs = p.findall('./w:r', NS)
                    if runs:
                        first_run = runs[0]
                        # Очищаем текст
                        for t in first_run.findall('./w:t', NS):
                            first_run.remove(t)
                        # Удаляем лишние runs
                        for r_elem in runs[1:]:
                            p.remove(r_elem)
                        # Вставляем новый текст (с preserve, чтобы не съедались пробелы)
                        t_elem = ET.SubElement(first_run, f"{{{NS['w']}}}t")
                        t_elem.set("{http://www.w3.org/XML/1998/namespace}space", "preserve")
                        t_elem.text = str(row_data[col_idx])
                col_idx += 1
        table.append(nr)

    tree.write(doc_xml, encoding='utf-8', xml_declaration=True)

    with zipfile.ZipFile(output, 'w', zipfile.ZIP_DEFLATED) as z:
        for f in tmp.rglob('*'):
            if f.is_file():
                z.write(f, f.relative_to(tmp))
    shutil.rmtree(tmp)

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument("template")
    parser.add_argument("output")
    parser.add_argument("data")
    parser.add_argument("--skip-header", action="store_true")
    parser.add_argument("--table-index", type=int, default=0)
    args = parser.parse_args()
    fill_table(args.template, args.output, args.data, args.skip_header, args.table_index)