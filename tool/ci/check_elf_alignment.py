#!/usr/bin/env python3
"""Fail if any native library in an APK/AAB/folder has LOAD segments aligned
to less than 16 KiB (spec section 2: 16 KB page-size support).

usage: check_elf_alignment.py <file.apk|file.aab|directory> [...]
"""
import io
import pathlib
import struct
import sys
import zipfile

MIN_ALIGN = 16 * 1024


def load_aligns(data: bytes):
    if data[:4] != b"\x7fELF":
        return None
    is64 = data[4] == 2
    end = "<" if data[5] == 1 else ">"
    if is64:
        phoff, = struct.unpack_from(end + "Q", data, 0x20)
        phentsize, phnum = struct.unpack_from(end + "HH", data, 0x36)
    else:
        phoff, = struct.unpack_from(end + "I", data, 0x1C)
        phentsize, phnum = struct.unpack_from(end + "HH", data, 0x2A)
    out = []
    for i in range(phnum):
        off = phoff + i * phentsize
        p_type, = struct.unpack_from(end + "I", data, off)
        if p_type != 1:  # PT_LOAD
            continue
        if is64:
            align, = struct.unpack_from(end + "Q", data, off + 48)
        else:
            align, = struct.unpack_from(end + "I", data, off + 28)
        out.append(align)
    return out


def check_blob(name: str, data: bytes) -> list:
    aligns = load_aligns(data)
    if aligns is None:
        return []
    if not aligns or min(aligns) < MIN_ALIGN:
        return [f"{name}: LOAD align {min(aligns) if aligns else 'none'} < {MIN_ALIGN}"]
    return []


def main(args) -> int:
    problems, count = [], 0
    for a in args:
        p = pathlib.Path(a)
        if p.is_dir():
            for f in p.rglob("*.so"):
                count += 1
                problems += check_blob(str(f), f.read_bytes())
        elif zipfile.is_zipfile(p):
            with zipfile.ZipFile(p) as z:
                for n in z.namelist():
                    if n.endswith(".so"):
                        count += 1
                        problems += check_blob(f"{p.name}!{n}", z.read(n))
        else:
            count += 1
            problems += check_blob(str(p), p.read_bytes())
    print(f"checked {count} native libraries")
    if problems:
        print("NOT 16 KB aligned:\n  " + "\n  ".join(problems))
        return 1
    print("check_elf_alignment: OK")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
