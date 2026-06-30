#!/usr/bin/env bash
set -euo pipefail

assert_wsta_makepad_is_wasm_safe() {
  if grep -Eq '^[[:space:]]*dsta[[:space:]]*=' wsta_makepad/Cargo.toml; then
    echo "ERROR: wsta_makepad must not depend on dsta; dsta pulls backend deps into wasm."
    exit 1
  fi

  if grep -Eq '^[[:space:]]*(ttrs|openssl|openssl-sys|quinn|rustls|wasm-bindgen|wasm_bindgen)[[:space:]]*=' wsta_makepad/Cargo.toml; then
    echo "ERROR: wsta_makepad has backend/native/wasm-bindgen deps that do not belong in the pure Makepad frontend."
    exit 1
  fi
}

assert_wsta_makepad_makepad_api_pin() {
  if grep -q 'makepad-widgets.*git' wsta_makepad/Cargo.toml; then
    echo "ERROR: wsta_makepad is pinned to released makepad-widgets API; do not use git master here."
    exit 1
  fi
}

write_cargo_wasm_config() {
  mkdir -p .cargo
  mkdir -p wsta_makepad/.cargo
  rm -f .cargo/config
  rm -f wsta_makepad/.cargo/config

  python - <<'PY'
from pathlib import Path

def write_config(path: str):
    p = Path(path)
    s = p.read_text() if p.exists() else ""

    section = "[target.wasm32-unknown-unknown]"
    rustflags = 'rustflags = ["-C", "link-args=--allow-undefined", "-C", "link-arg=--export=__stack_pointer"]'

    out = []
    lines = s.splitlines()
    i = 0

    while i < len(lines):
        line = lines[i]
        if line.strip() == section:
            i += 1
            while i < len(lines) and not (lines[i].strip().startswith("[") and lines[i].strip().endswith("]")):
                i += 1
            continue
        out.append(line)
        i += 1

    while out and out[-1].strip() == "":
        out.pop()

    if out:
        out.append("")
    out.append(section)
    out.append(rustflags)
    out.append("")

    p.write_text("\n".join(out))

write_config(".cargo/config.toml")
write_config("wsta_makepad/.cargo/config.toml")
PY
}

hard_force_makepad_wasm_rustflags() {
  local sep
  sep="$(printf '\037')"

  export CARGO_ENCODED_RUSTFLAGS="-Clink-args=--allow-undefined${sep}-Clink-arg=--export=__stack_pointer"
  export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS="-C link-args=--allow-undefined -C link-arg=--export=__stack_pointer"
  export RUSTFLAGS="-C link-args=--allow-undefined -C link-arg=--export=__stack_pointer"

  echo "CARGO_ENCODED_RUSTFLAGS is set for Makepad WASM link"
  echo "CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS=${CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS}"
  echo "RUSTFLAGS=${RUSTFLAGS}"
}

patch_local_makepad_cargo_tool() {
  while IFS= read -r mod_rs; do
    python - "$mod_rs" <<'PY2'
from pathlib import Path
import sys

p = Path(sys.argv[1])
s = p.read_text()

linux_line = '    #[cfg(all(target_os = "linux"))]\n    let host_os = HostOs::Linux;\n'

if 'target_os = "linux"' in s and 'HostOs::Linux' in s:
    raise SystemExit(0)

mac_line = '    #[cfg(all(target_os = "macos"))]\n    let host_os = HostOs::MacOS;\n'

if mac_line in s:
    s = s.replace(mac_line, mac_line + linux_line)
else:
    win_line = '    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]\n    let host_os = HostOs::WindowsX64;\n'
    if win_line in s:
        s = s.replace(win_line, win_line + linux_line)
    else:
        marker = 'pub fn handle_open_harmony(mut args: &[String]) -> Result<(), String> {'
        if marker not in s:
            raise SystemExit(f"could not find OpenHarmony insertion marker in {p}")
        s = s.replace(marker, marker + "\n" + linux_line, 1)

p.write_text(s)
PY2
  done < <(
    find "$HOME/.cargo/git/checkouts" \
      -path '*/tools/cargo_makepad/src/open_harmony/mod.rs' \
      -type f \
      -print 2>/dev/null \
    | sort
  )
}

ensure_cargo_makepad() {
  if command -v cargo-makepad >/dev/null 2>&1 || cargo makepad --version >/dev/null 2>&1; then
    return
  fi

  echo "cargo makepad is not installed. Installing from checked-out Makepad git dependency source if available."

  TOOL_DIR="$(
    find "$HOME/.cargo/git/checkouts" \
      -path '*/tools/cargo_makepad/Cargo.toml' \
      -type f \
      -printf '%T@ %h\n' 2>/dev/null \
    | sort -nr \
    | head -n 1 \
    | cut -d' ' -f2-
  )"

  if [[ -z "${TOOL_DIR:-}" || ! -f "$TOOL_DIR/Cargo.toml" ]]; then
    echo "ERROR: could not find tools/cargo_makepad checkout."
    exit 1
  fi

  cargo install --path "$TOOL_DIR" --force
}

build_makepad_package() {
  echo "Installing Rust toolchains for wasm"
  cargo makepad wasm install-toolchain

  echo "Building Makepad WASM package, release first"
  if cargo makepad wasm build -p wsta_makepad --release; then
    echo "MAKEPAD_BUILD_PROFILE=release"
  else
    echo "WARNING: release Makepad build failed; falling back to debug for diagnostics."
    cargo makepad wasm build -p wsta_makepad
    echo "MAKEPAD_BUILD_PROFILE=debug"
  fi
}

find_makepad_package_dir() {
  local preferred

  for preferred in \
    "target/makepad-wasm-app/release/wsta_makepad" \
    "target/makepad-wasm-app/debug/wsta_makepad"
  do
    if [[ -f "$preferred/index.html" ]]; then
      echo "$preferred"
      return 0
    fi
  done

  preferred="$(
    find target/makepad-wasm-app -path '*/wsta_makepad/index.html' -type f 2>/dev/null \
      | sed 's#/index.html$##' \
      | sort \
      | tail -n 1 \
      || true
  )"

  if [[ -n "${preferred:-}" && -f "$preferred/index.html" ]]; then
    echo "$preferred"
    return 0
  fi

  return 1
}

copy_makepad_dependency_resources() {
  local dst_pkg="$1"

  echo "== deploying lean Makepad dependency resources =="
  echo "Chinese/emoji URL paths are satisfied by small IBM Plex fallback TTFs, not huge glyph payloads."

  python - "$dst_pkg" <<'PY2'
from pathlib import Path
import shutil
import sys

dst = Path(sys.argv[1])
search_roots = [
    Path.home() / ".cargo" / "registry" / "src",
    Path.home() / ".cargo" / "git" / "checkouts",
]

copied = []

def find_file(name: str):
    for root in search_roots:
        if not root.exists():
            continue
        matches = list(root.rglob(name))
        if matches:
            matches.sort(key=lambda p: (0 if "makepad-widgets" in str(p) or "makepad_widgets" in str(p) else 1, len(str(p))))
            return matches[0]
    return None

def copy_to(src: Path, rel: str):
    out = dst / rel
    out.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, out)
    copied.append(str(out))

ibm_text = find_file("IBMPlexSans-Text.ttf")
ibm_semibold = find_file("IBMPlexSans-SemiBold.ttf") or ibm_text
ibm_bold_italic = find_file("IBMPlexSans-BoldItalic.ttf") or ibm_text
ibm_italic = find_file("IBMPlexSans-Italic.ttf") or ibm_text

if not ibm_text:
    raise SystemExit("ERROR: could not find IBMPlexSans-Text.ttf in Cargo registry/git checkouts")

copy_to(ibm_text, "makepad_widgets/resources/IBMPlexSans-Text.ttf")
if ibm_semibold:
    copy_to(ibm_semibold, "makepad_widgets/resources/IBMPlexSans-SemiBold.ttf")
if ibm_bold_italic:
    copy_to(ibm_bold_italic, "makepad_widgets/resources/IBMPlexSans-BoldItalic.ttf")
if ibm_italic:
    copy_to(ibm_italic, "makepad_widgets/resources/IBMPlexSans-Italic.ttf")

copy_to(ibm_text, "makepad_fonts_chinese_regular/resources/LXGWWenKaiRegular.ttf")
copy_to(ibm_text, "makepad_fonts_chinese_regular/resources/LXGWWenKaiRegular.ttf.2")
copy_to(ibm_text, "makepad_fonts_chinese_regular_2/resources/LXGWWenKaiRegular.ttf.2")
copy_to(ibm_semibold or ibm_text, "makepad_fonts_chinese_bold/resources/LXGWWenKaiBold.ttf")
copy_to(ibm_semibold or ibm_text, "makepad_fonts_chinese_bold/resources/LXGWWenKaiBold.ttf.2")
copy_to(ibm_text, "makepad_fonts_emoji/resources/NotoColorEmoji.ttf")

print(f"copied {len(copied)} lean Makepad resource files")
for f in copied:
    print("  " + f)
PY2
}

inject_wsta_browser_profile_probe() {
  local dst_pkg="$1"
  local index="$dst_pkg/index.html"

  echo "== injecting WSTA browser profiling/debug probe into generated index.html =="

  python - "$index" <<'PY2'
from pathlib import Path
import sys

p = Path(sys.argv[1])
s = p.read_text()

if "WSTA_PROFILE_PROBE_V1" in s:
    raise SystemExit(0)

probe = r'''
<script>
/* WSTA_PROFILE_PROBE_V1 */
(() => {
  const t0 = performance.now();
  const marks = [];

  function mark(name) {
    const t = performance.now();
    const row = { name, ms: Math.round((t - t0) * 10) / 10 };
    marks.push(row);
    console.log("[WSTA profile]", row.name, row.ms + "ms");
    draw();
  }

  function report(kind, data) {
    try {
      const payload = encodeURIComponent(JSON.stringify({
        kind,
        href: location.href,
        user_agent: navigator.userAgent,
        cross_origin_isolated: window.crossOriginIsolated,
        data,
        marks,
      }));
      fetch("/$report_error?data=" + payload, { cache: "no-store" }).catch(() => {});
    } catch (_) {}
  }

  function draw() {
    let el = document.getElementById("wsta_profile_overlay");
    if (!el) {
      el = document.createElement("div");
      el.id = "wsta_profile_overlay";
      el.style.cssText = [
        "position:fixed",
        "right:8px",
        "bottom:8px",
        "z-index:999999",
        "max-width:520px",
        "font:11px ui-monospace,monospace",
        "color:#70d6ff",
        "background:rgba(0,0,0,0.72)",
        "border:1px solid rgba(112,214,255,0.35)",
        "border-radius:6px",
        "padding:6px 8px",
        "white-space:pre-wrap",
        "pointer-events:none"
      ].join(";");
      document.addEventListener("DOMContentLoaded", () => document.body.appendChild(el));
      if (document.body) document.body.appendChild(el);
    }

    const latest = marks.slice(-8).map(m => `${m.ms}ms ${m.name}`).join("\\n");
    el.textContent =
      `WSTA profile\\n` +
      `isolated=${window.crossOriginIsolated}\\n` +
      `threads=${typeof Worker !== "undefined" ? "worker-api" : "no-worker-api"}\\n` +
      latest;
  }

  window.addEventListener("error", ev => {
    report("window.error", {
      message: ev.message,
      filename: ev.filename,
      lineno: ev.lineno,
      colno: ev.colno,
      stack: ev.error && ev.error.stack ? ev.error.stack : "",
    });
  });

  window.addEventListener("unhandledrejection", ev => {
    report("window.unhandledrejection", {
      reason_message: ev.reason && ev.reason.message ? ev.reason.message : String(ev.reason),
      reason_stack: ev.reason && ev.reason.stack ? ev.reason.stack : "",
    });
  });

  mark("profile probe installed");

  document.addEventListener("DOMContentLoaded", () => mark("DOMContentLoaded"));
  window.addEventListener("load", () => {
    mark("window load");
    setTimeout(() => {
      mark("load+1000");
      report("profile.snapshot", {
        cross_origin_isolated: window.crossOriginIsolated,
        worker_available: typeof Worker !== "undefined",
        shared_array_buffer_available: typeof SharedArrayBuffer !== "undefined",
        resource_count: performance.getEntriesByType("resource").length,
        resources: performance.getEntriesByType("resource").map(r => ({
          name: r.name,
          initiatorType: r.initiatorType,
          duration: Math.round(r.duration),
          transferSize: r.transferSize || 0,
          encodedBodySize: r.encodedBodySize || 0,
        })).slice(-80),
      });
    }, 1000);
  });
})();
</script>
'''

if "</head>" in s:
    s = s.replace("</head>", probe + "\n</head>", 1)
else:
    s = probe + "\n" + s

p.write_text(s)
PY2
}

deploy_makepad_package() {
  local src_pkg
  local dst_pkg

  src_pkg="$(find_makepad_package_dir)" || {
    echo "ERROR: Makepad build finished but no generated package root with index.html was found."
    find target/makepad-wasm-app -maxdepth 6 -type f | sort 2>/dev/null || true
    exit 1
  }

  dst_pkg="wsta_makepad/target/makepad-wasm"

  echo "== deploying Makepad package =="
  echo "source: $src_pkg"
  echo "dest:   $dst_pkg"

  rm -rf "$dst_pkg"
  mkdir -p "$dst_pkg"

  if command -v rsync >/dev/null 2>&1; then
    rsync -a "$src_pkg"/ "$dst_pkg"/
  else
    cp -a "$src_pkg"/. "$dst_pkg"/
  fi

  mkdir -p "$dst_pkg/resources/web"
  if [[ -f "wsta_makepad/resources/web/wsta_transport.js" ]]; then
    cp "wsta_makepad/resources/web/wsta_transport.js" "$dst_pkg/resources/web/wsta_transport.js"
  fi

  copy_makepad_dependency_resources "$dst_pkg"
  inject_wsta_browser_profile_probe "$dst_pkg"

  if [[ ! -f "$dst_pkg/index.html" ]]; then
    echo "ERROR: deployed Makepad package has no index.html."
    find "$dst_pkg" -maxdepth 5 -type f | sort
    exit 1
  fi

  echo "== deployed Makepad package key files =="
  find "$dst_pkg" -maxdepth 5 -type f | sort | grep -E 'index.html|\.wasm$|wasm_bridge.js|web.js|LXGWWenKaiRegular|NotoColorEmoji|IBMPlexSans' || true
}

assert_wsta_makepad_is_wasm_safe
assert_wsta_makepad_makepad_api_pin
patch_local_makepad_cargo_tool
write_cargo_wasm_config
hard_force_makepad_wasm_rustflags
ensure_cargo_makepad
build_makepad_package
deploy_makepad_package

echo
echo "DONE."
echo "WSTA Makepad package is deployed at wsta_makepad/target/makepad-wasm/index.html"
echo "Run cargo run -p wsta and hard-refresh http://127.0.0.1:8088/"
