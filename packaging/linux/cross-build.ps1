# Windows cross-build using the Zig cc/ar wrappers and Debian linker library
# prepared in Tools. Use a short temporary path for the Zig response file.
param([Parameter(Mandatory)][string]$Tools)
$ErrorActionPreference='Stop'
$root=(Resolve-Path "$PSScriptRoot\..\..").Path
$toolsDir=(Resolve-Path $Tools).Path
$stage=Join-Path ([IO.Path]::GetTempPath()) ('bs-cross-'+[Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory $stage | Out-Null
Copy-Item -LiteralPath "$root\src" -Destination "$stage\src" -Recurse
Copy-Item -LiteralPath "$root\Cargo.toml","$root\Cargo.lock","$toolsDir\libdbus-1.so" -Destination $stage
$dirUnix=$stage.Replace('\','/')
@"
[target.x86_64-unknown-linux-gnu.dbus]
rustc-link-lib=["dbus-1"]
rustc-link-search=["native=$dirUnix"]
"@ | Set-Content "$stage\linux.toml"
$env:CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$toolsDir\linux-cc.exe"
$env:CC_x86_64_unknown_linux_gnu=$env:CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER
$env:AR_x86_64_unknown_linux_gnu="$toolsDir\linux-ar.exe"
$env:ZIG_GLOBAL_CACHE_DIR="$stage\zig-cache"
Push-Location -LiteralPath $stage
try {
    cargo --config linux.toml build --offline --locked --release --target x86_64-unknown-linux-gnu *> "$toolsDir\linux-final.log"
    if($LASTEXITCODE -ne 0){Get-Content "$toolsDir\linux-final.log" -Tail 15; throw 'Linux build failed'}
    $out=New-Item -ItemType Directory -Force "$root\target\x86_64-unknown-linux-gnu\release"
    Copy-Item target\x86_64-unknown-linux-gnu\release\basestation-tool $out.FullName
} finally { Pop-Location }
