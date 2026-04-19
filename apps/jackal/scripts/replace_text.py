#!/usr/bin/env python3
import sys, json, zipfile, re, shutil
from pathlib import Path
import tempfile
import xml.etree.ElementTree as ET

NS = {'w': 'http://schemas.openxmlformats.org/wordprocessingml/2006/main'}
ET.register_namespace('w', NS['w'])

def replace_text(docx_path, replacements_json):
    with open(replacements_json, 'r', encoding='utf-8') as f:
        replacements = json.load(f)

    tmp = Path(tempfile.mkdtemp())
    with zipfile.ZipFile(docx_path, 'r') as z:
        z.extractall(tmp)

    xml_files = list(tmp.rglob('*.xml'))

    for xml_file in xml_files:
        tree = ET.parse(xml_file)
        root = tree.getroot()
        modified = False

        for repl in replacements:
            find_str = repl['find']
            replace_str = repl.get('replace', '')
            whole_word = repl.get('wholeWord', False)
            case_sensitive = repl.get('caseSensitive', False)

            for t_elem in root.findall('.//w:t', NS):
                if not t_elem.text:
                    continue

                old = t_elem.text
                new = old

                if not case_sensitive:
                    if whole_word:
                        words = re.split(r'(\W+)', old)
                        for i, w in enumerate(words):
                            if w.lower() == find_str.lower():
                                words[i] = replace_str
                        new = ''.join(words)
                    else:
                        pattern = re.compile(re.escape(find_str), re.IGNORECASE)
                        new = pattern.sub(replace_str, old)
                else:
                    if whole_word:
                        words = re.split(r'(\W+)', old)
                        for i, w in enumerate(words):
                            if w == find_str:
                                words[i] = replace_str
                        new = ''.join(words)
                    else:
                        new = old.replace(find_str, replace_str)

                if new != old:
                    t_elem.text = new
                    modified = True

        if modified:
            tree.write(xml_file, encoding='utf-8', xml_declaration=True)

    with zipfile.ZipFile(docx_path, 'w', zipfile.ZIP_DEFLATED) as z:
        for f in tmp.rglob('*'):
            if f.is_file():
                z.write(f, f.relative_to(tmp))
    shutil.rmtree(tmp)

if __name__ == '__main__':
    if len(sys.argv) != 3:
        sys.exit("Usage: replace_text.py document.docx replacements.json")
    replace_text(sys.argv[1], sys.argv[2])