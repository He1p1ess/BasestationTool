"""Build and verify a type-2 AppImage; requires the official x86-64 runtime
and tar2sqfs/sqfs2tar from squashfs-tools-ng. No mount or root access needed.
"""
import argparse
import io
from pathlib import Path
import struct
import subprocess
import tarfile
import tempfile
import tomllib

p = argparse.ArgumentParser()
for arg in ("binary", "runtime", "out"):
    p.add_argument(arg, type=Path)
p.add_argument("--packer", default="tar2sqfs")
p.add_argument("--unpacker", default="sqfs2tar")
a = p.parse_args()
version = tomllib.loads((Path(__file__).resolve().parents[2] / "Cargo.toml").read_text())["package"]["version"]
runtime, binary = a.runtime.read_bytes(), a.binary.read_bytes()
for elf in (runtime, binary):
    if elf[:6] != b"\x7fELF\x02\x01" or struct.unpack_from("<H", elf, 18)[0] != 62:
        p.error("Expected Linux x86-64 ELF files")
offset = struct.unpack_from("<Q", runtime, 40)[0] + (
    struct.unpack_from("<H", runtime, 58)[0] * struct.unpack_from("<H", runtime, 60)[0])
if runtime[8:11] != b"AI\x02" or offset != len(runtime):
    p.error("Expected an unappended type-2 runtime")

desktop = f"""[Desktop Entry]
Type=Application
Name=Basestation Tool
Comment=Manage Lighthouse base stations over Bluetooth
Exec=basestation-tool
Icon=basestation-tool
Terminal=false
Categories=Utility;
X-AppImage-Version={version}
""".encode()
icon = b"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128">
<rect x="8" y="8" width="112" height="112" rx="26" fill="#17232e"/>
<rect x="28" y="25" width="72" height="78" rx="14" fill="#34475b"/>
<rect x="35" y="37" width="58" height="44" rx="10" fill="#10191f"/>
<circle cx="64" cy="59" r="10" fill="#68dcc5"/>
<circle cx="85" cy="92" r="4" fill="#96de89"/></svg>"""
exe = "usr/bin/basestation-tool"
payload = {exe: (binary, 0o755), "basestation-tool.desktop": (desktop, 0o644),
           "basestation-tool.svg": (icon, 0o644)}
links = {"AppRun": exe, ".DirIcon": "basestation-tool.svg"}
with tempfile.TemporaryDirectory() as tmp:
    squash = Path(tmp) / "app.squashfs"
    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w", format=tarfile.USTAR_FORMAT) as tar:
        for name, (data, mode) in payload.items():
            entry = tarfile.TarInfo(name)
            entry.mode, entry.size = mode, len(data)
            tar.addfile(entry, io.BytesIO(data))
        for name, target in links.items():
            entry = tarfile.TarInfo(name)
            entry.type, entry.mode, entry.linkname = tarfile.SYMTYPE, 0o777, target
            tar.addfile(entry)
    subprocess.run([a.packer, "-c", "gzip", "-s", squash], input=buf.getvalue(), check=True)
    candidate = Path(tmp) / "candidate.AppImage"
    candidate.write_bytes(runtime + squash.read_bytes())
    # Extract from the assembled image, at the offset derived from its ELF.
    with candidate.open("rb") as f:
        assert f.read(offset) == runtime
        squash.write_bytes(f.read())
    decoded = subprocess.check_output([a.unpacker, squash])
    with tarfile.open(fileobj=io.BytesIO(decoded)) as tar:
        for name, (data, mode) in payload.items():
            if tar.extractfile(name).read() != data or tar.getmember(name).mode != mode:
                raise ValueError(f"Payload mismatch: {name}")
        for name, target in links.items():
            entry = tar.getmember(name)
            if not entry.issym() or entry.linkname != target:
                raise ValueError(f"Broken link: {name}")
    a.out.mkdir(exist_ok=True, parents=True)
    output = a.out / f"Basestation-Tool-{version}-x86_64.AppImage"
    output.write_bytes(candidate.read_bytes())
    output.chmod(0o755)
    print(f"Verified {output} ({output.stat().st_size} bytes)")
