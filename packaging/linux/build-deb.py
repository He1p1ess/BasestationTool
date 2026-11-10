"""Pack a compiled Linux x86-64 ELF into a Debian binary package (Python stdlib)."""
import argparse
import gzip
import io
from pathlib import Path
import struct
import tarfile
import tomllib

parser = argparse.ArgumentParser()
parser.add_argument("binary", type=Path)
parser.add_argument("out", type=Path)
args = parser.parse_args()
version = tomllib.loads((Path(__file__).resolve().parents[2] / "Cargo.toml").read_text())["package"]["version"]
elf = args.binary.read_bytes()
if elf[:6] != b"\x7fELF\x02\x01" or struct.unpack_from("<H", elf, 18)[0] != 62:
    parser.error("Expected a 64-bit little-endian x86-64 ELF binary")

def archive(items):
    data = io.BytesIO()
    with tarfile.open(fileobj=data, mode="w", format=tarfile.USTAR_FORMAT) as tar:
        for name, value, mode in items:
            entry = tarfile.TarInfo(name)
            entry.size, entry.mode = len(value), mode
            entry.uname = entry.gname = "root"
            tar.addfile(entry, io.BytesIO(value))
    return gzip.compress(data.getvalue(), mtime=0)

desktop = b"""[Desktop Entry]
Type=Application
Name=Basestation Tool
Comment=Manage Lighthouse base stations over Bluetooth
Exec=basestation-tool
Icon=preferences-system-bluetooth
Terminal=false
Categories=Utility;
"""
control = f"""Package: basestation-tool
Version: {version}
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Basestation Tool contributors
Depends: libc6 (>= 2.31), libgcc-s1, libdbus-1-3, libxkbcommon0, libx11-6, libxcursor1, libxi6, libxrandr2, libegl1, libgl1, libwayland-client0, bluez, xdg-utils
Description: Bluetooth controller for Lighthouse base stations
 Native desktop application for scanning and controlling Lighthouse stations.
""".encode()
members = [("debian-binary", b"2.0\n"), ("control.tar.gz", archive([("./control", control, 0o644)])),
           ("data.tar.gz", archive([("./usr/bin/basestation-tool", elf, 0o755),
                                    ("./usr/share/applications/basestation-tool.desktop", desktop, 0o644)]))]
args.out.mkdir(parents=True, exist_ok=True)
with (args.out / f"basestation-tool_{version}_amd64.deb").open("wb") as pkg:
    pkg.write(b"!<arch>\n")
    for name, content in members:
        pkg.write(f"{name+'/':16}{0:<12}{0:<6}{0:<6}{'100644':8}{len(content):<10}`\n".encode())
        pkg.write(content)
        if len(content) % 2:
            pkg.write(b"\n")
