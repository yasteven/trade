#!/usr/bin/env bash
set -euo pipefail

echo "== building WSTA Makepad WASM: THREADED =="
echo "Uses Makepad's own threaded flag/export contract from cargo_makepad/src/wasm/compile.rs."
echo "If the final wasm is missing required thread exports, this script fails before deployment."

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

write_cargo_wasm_config_threaded() {
  mkdir -p .cargo
  mkdir -p wsta_makepad/.cargo
  rm -f .cargo/config
  rm -f wsta_makepad/.cargo/config

  python - <<'PY'
from pathlib import Path

rustflags = 'rustflags = ["-C", "codegen-units=1", "-C", "debuginfo=0", "-C", "target-feature=+atomics,+bulk-memory,+mutable-globals", "-C", "link-arg=--export=__stack_pointer", "-C", "link-arg=--compress-relocations", "-C", "link-arg=--strip-debug", "-C", "link-arg=--shared-memory", "-C", "link-arg=--max-memory=2147483648", "-C", "link-arg=--import-memory", "-C", "link-arg=--export=__wasm_init_tls", "-C", "link-arg=--export=__tls_size", "-C", "link-arg=--export=__tls_align", "-C", "link-arg=--export=__tls_base", "-C", "link-args=--allow-undefined", "-C", "opt-level=z"]'

def write_config(path: str):
    p = Path(path)
    s = p.read_text() if p.exists() else ""
    lines = s.splitlines()
    out = []
    i = 0
    while i < len(lines):
        if lines[i].strip() == "[target.wasm32-unknown-unknown]":
            i += 1
            while i < len(lines) and not (lines[i].strip().startswith("[") and lines[i].strip().endswith("]")):
                i += 1
            continue
        out.append(lines[i])
        i += 1
    while out and out[-1].strip() == "":
        out.pop()
    if out:
        out.append("")
    out.append("[target.wasm32-unknown-unknown]")
    out.append(rustflags)
    out.append("")
    p.write_text("\n".join(out))

write_config(".cargo/config.toml")
write_config("wsta_makepad/.cargo/config.toml")
PY
}

force_threaded_rustflags() {
  local sep
  sep="$(printf '\037')"

  export CARGO_ENCODED_RUSTFLAGS="-Ccodegen-units=1${sep}-Cdebuginfo=0${sep}-Ctarget-feature=+atomics,+bulk-memory,+mutable-globals${sep}-Clink-arg=--export=__stack_pointer${sep}-Clink-arg=--compress-relocations${sep}-Clink-arg=--strip-debug${sep}-Clink-arg=--shared-memory${sep}-Clink-arg=--max-memory=2147483648${sep}-Clink-arg=--import-memory${sep}-Clink-arg=--export=__wasm_init_tls${sep}-Clink-arg=--export=__tls_size${sep}-Clink-arg=--export=__tls_align${sep}-Clink-arg=--export=__tls_base${sep}-Clink-args=--allow-undefined${sep}-Copt-level=z"
  export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS="-C codegen-units=1 -C debuginfo=0 -C target-feature=+atomics,+bulk-memory,+mutable-globals -C link-arg=--export=__stack_pointer -C link-arg=--compress-relocations -C link-arg=--strip-debug -C link-arg=--shared-memory -C link-arg=--max-memory=2147483648 -C link-arg=--import-memory -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base -C link-args=--allow-undefined -C opt-level=z"
  export RUSTFLAGS="${CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS}"

  echo "threaded RUSTFLAGS:"
  echo "${RUSTFLAGS}"
}

ensure_cargo_makepad() {
  if command -v cargo-makepad >/dev/null 2>&1 || cargo makepad --version >/dev/null 2>&1; then
    return
  fi

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

build_threaded_package() {
  echo "Installing Rust toolchains for wasm"
  cargo makepad wasm install-toolchain

  echo "Building threaded Makepad WASM package"
  cargo makepad wasm build -p wsta_makepad --release
}

find_makepad_package_dir() {
  for preferred in \
    "target/makepad-wasm-app/release/wsta_makepad" \
    "target/makepad-wasm-app/debug/wsta_makepad"
  do
    if [[ -f "$preferred/index.html" ]]; then
      echo "$preferred"
      return 0
    fi
  done

  find target/makepad-wasm-app -path '*/wsta_makepad/index.html' -type f 2>/dev/null \
    | sed 's#/index.html$##' \
    | sort \
    | tail -n 1
}

copy_lean_fallback_fonts() {
  local dst_pkg="$1"

  echo "== deploying lean valid font fallbacks =="

  python - "$dst_pkg" <<'PY'
from pathlib import Path
import shutil
import sys

dst = Path(sys.argv[1])
roots = [Path.home()/".cargo"/"registry"/"src", Path.home()/".cargo"/"git"/"checkouts"]

def find_file(name):
    for root in roots:
        if not root.exists():
            continue
        matches = list(root.rglob(name))
        if matches:
            matches.sort(key=lambda p: (0 if "makepad-widgets" in str(p) or "makepad_widgets" in str(p) else 1, len(str(p))))
            return matches[0]
    return None

def copy_to(src, rel):
    out = dst / rel
    out.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, out)
    print("  " + str(out))

text = find_file("IBMPlexSans-Text.ttf")
semi = find_file("IBMPlexSans-SemiBold.ttf") or text
bold_italic = find_file("IBMPlexSans-BoldItalic.ttf") or text
italic = find_file("IBMPlexSans-Italic.ttf") or text

if not text:
    raise SystemExit("ERROR: could not find IBMPlexSans-Text.ttf")

copy_to(text, "makepad_widgets/resources/IBMPlexSans-Text.ttf")
copy_to(semi, "makepad_widgets/resources/IBMPlexSans-SemiBold.ttf")
copy_to(bold_italic, "makepad_widgets/resources/IBMPlexSans-BoldItalic.ttf")
copy_to(italic, "makepad_widgets/resources/IBMPlexSans-Italic.ttf")

# Valid small fallback bytes for stock Makepad optional paths.
copy_to(text, "makepad_fonts_chinese_regular/resources/LXGWWenKaiRegular.ttf")
copy_to(text, "makepad_fonts_chinese_regular/resources/LXGWWenKaiRegular.ttf.2")
copy_to(text, "makepad_fonts_chinese_regular_2/resources/LXGWWenKaiRegular.ttf.2")
copy_to(semi, "makepad_fonts_chinese_bold/resources/LXGWWenKaiBold.ttf")
copy_to(semi, "makepad_fonts_chinese_bold/resources/LXGWWenKaiBold.ttf.2")
copy_to(text, "makepad_fonts_emoji/resources/NotoColorEmoji.ttf")
PY
}

inject_wsta_browser_profile_probe() {
  local dst_pkg="$1"
  local index="$dst_pkg/index.html"

  if grep -q "WSTA_PROFILE_PROBE_V1" "$index"; then
    return
  fi

  python - "$index" <<'PY'
from pathlib import Path
import sys

p = Path(sys.argv[1])
s = p.read_text()
probe = r'''
<script>
/* WSTA_PROFILE_PROBE_V1 */
(() => {
  const t0 = performance.now();
  const marks = [];
  function mark(name) {
    const ms = Math.round((performance.now() - t0) * 10) / 10;
    marks.push({name, ms});
    console.log("[WSTA profile]", name, ms + "ms");
    draw();
  }
  function draw() {
    let el = document.getElementById("wsta_profile_overlay");
    if (!el) {
      el = document.createElement("div");
      el.id = "wsta_profile_overlay";
      el.style.cssText = "position:fixed;right:8px;bottom:8px;z-index:999999;max-width:620px;font:11px ui-monospace,monospace;color:#70d6ff;background:rgba(0,0,0,0.72);border:1px solid rgba(112,214,255,0.35);border-radius:6px;padding:6px 8px;white-space:pre-wrap;pointer-events:none";
      if (document.body) document.body.appendChild(el);
      else document.addEventListener("DOMContentLoaded", () => document.body.appendChild(el));
    }
    el.textContent = "WSTA profile\\n" +
      "isolated=" + window.crossOriginIsolated + "\\n" +
      "threads=" + (typeof Worker !== "undefined" ? "worker-api" : "no-worker-api") + "\\n" +
      marks.slice(-8).map(m => `${m.ms}ms ${m.name}`).join("\\n");
  }
  function report(kind, data) {
    try {
      fetch("/$report_error?data=" + encodeURIComponent(JSON.stringify({
        kind, href: location.href, user_agent: navigator.userAgent,
        cross_origin_isolated: window.crossOriginIsolated, data, marks
      })), {cache:"no-store"}).catch(()=>{});
    } catch (_) {}
  }
  window.addEventListener("error", ev => report("window.error", {
    message: ev.message, filename: ev.filename, lineno: ev.lineno, colno: ev.colno,
    stack: ev.error && ev.error.stack ? ev.error.stack : ""
  }));
  window.addEventListener("unhandledrejection", ev => report("window.unhandledrejection", {
    reason_message: ev.reason && ev.reason.message ? ev.reason.message : String(ev.reason),
    reason_stack: ev.reason && ev.reason.stack ? ev.reason.stack : ""
  }));
  mark("profile probe installed");
  document.addEventListener("DOMContentLoaded", () => mark("DOMContentLoaded"));
  window.addEventListener("load", () => setTimeout(() => mark("load+1000"), 1000));
})();
</script>
'''
if "</head>" in s:
    s = s.replace("</head>", probe + "\\n</head>", 1)
else:
    s = probe + "\\n" + s
p.write_text(s)
PY
}

verify_threaded_exports() {
  local wasm="$1"

  echo "== verifying threaded wasm exports with built-in parser =="

  python - "$wasm" <<'PY2'
from pathlib import Path
import sys

wasm = Path(sys.argv[1])
data = wasm.read_bytes()

if data[:4] != b"\0asm":
    raise SystemExit(f"ERROR: not a wasm module: {wasm}")

pos = 8  # magic + version

def read_u32_leb(data, pos):
    result = 0
    shift = 0
    while True:
        if pos >= len(data):
            raise ValueError("unexpected EOF while reading LEB128")
        b = data[pos]
        pos += 1
        result |= (b & 0x7f) << shift
        if (b & 0x80) == 0:
            return result, pos
        shift += 7
        if shift > 35:
            raise ValueError("LEB128 too large")

exports = set()

while pos < len(data):
    section_id = data[pos]
    pos += 1
    size, pos = read_u32_leb(data, pos)
    section_start = pos
    section_end = pos + size

    if section_id == 7:  # export section
        count, q = read_u32_leb(data, section_start)
        for _ in range(count):
            name_len, q = read_u32_leb(data, q)
            name = data[q:q + name_len].decode("utf-8", errors="replace")
            q += name_len
            kind = data[q]
            q += 1
            index, q = read_u32_leb(data, q)
            exports.add(name)
        break

    pos = section_end

required = [
    "__stack_pointer",
    "__wasm_init_tls",
    "__tls_size",
    "__tls_align",
    "__tls_base",
    "wasm_thread_alloc_tls_and_stack",
    "wasm_thread_entrypoint",
    "wasm_thread_timer_entrypoint",
]

missing = []
for sym in required:
    if sym in exports:
        print(f"ok: {sym}")
    else:
        print(f"ERROR: missing threaded wasm export: {sym}")
        missing.append(sym)

if missing:
    print()
    print("Threaded Makepad WASM build is incomplete. Refusing to deploy broken threaded wasm.")
    print()
    print("Available nearby exports:")
    for e in sorted(exports):
        if "tls" in e or "thread" in e or "stack" in e:
            print("  " + e)
    raise SystemExit(1)

print("threaded wasm export verification passed")
PY2
}

deploy_threaded_package() {
  local src_pkg
  src_pkg="$(find_makepad_package_dir)"

  if [[ -z "${src_pkg:-}" || ! -f "$src_pkg/index.html" ]]; then
    echo "ERROR: could not find Makepad package root with index.html"
    find target/makepad-wasm-app -maxdepth 6 -type f | sort 2>/dev/null || true
    exit 1
  fi

  local dst_pkg="wsta_makepad/target/makepad-wasm"

  echo "== deploying threaded Makepad package =="
  echo "source: $src_pkg"
  echo "dest:   $dst_pkg"

  rm -rf "$dst_pkg"
  mkdir -p "$dst_pkg"

  if command -v rsync >/dev/null 2>&1; then
    rsync -a "$src_pkg"/ "$dst_pkg"/
  else
    cp -a "$src_pkg"/. "$dst_pkg"/
  fi

  copy_lean_fallback_fonts "$dst_pkg"
  inject_wsta_browser_profile_probe "$dst_pkg"
  verify_threaded_exports "$dst_pkg/wsta_makepad.wasm"

  echo "== threaded package key files =="
  find "$dst_pkg" -maxdepth 5 -type f | sort | grep -E 'index.html|wsta_makepad.wasm|web.js|web_worker.js|wasm_bridge.js|IBMPlexSans|LXGWWenKai|NotoColorEmoji' || true
}

assert_wsta_makepad_is_wasm_safe
assert_wsta_makepad_makepad_api_pin
write_cargo_wasm_config_threaded
force_threaded_rustflags
ensure_cargo_makepad

rm -rf ./wsta_makepad/target/makepad-wasm
rm -rf ./target/makepad-wasm-app
rm -rf ./target/wasm32-unknown-unknown

build_threaded_package
deploy_threaded_package

echo
echo "DONE."
echo "Threaded Makepad WASM deployed."
echo "Run:"
echo "  cargo run -p wsta"
