#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="./info"
ALL_OUT="$OUT_DIR/cat_all_trade.txt"
ALL_RS_OUT="$OUT_DIR/cat_all_rs.txt"
ALL_TOML_OUT="$OUT_DIR/cat_all_toml.txt"

mkdir -p "$OUT_DIR"
: > "$ALL_OUT"
: > "$ALL_RS_OUT"
: > "$ALL_TOML_OUT"

append_file() {
    local out_file="$1"
    local path="$2"
    local label="$3"
    if [[ ! -f "$path" ]]; then
        return 0
    fi
    {
        echo
        echo "################################################################"
        echo "# $label"
        echo "# PATH: $path"
        echo "################################################################"
        echo
        cat "$path"
        echo
    } >> "$out_file"
}

append_tree_files() {
    local out_file="$1"
    local root="$2"
    local label="$3"
    if [[ ! -d "$root" ]]; then
        {
            echo
            echo "################################################################"
            echo "# $label"
            echo "# PATH: $root"
            echo "# NOTE: directory not found"
            echo "################################################################"
            echo
        } >> "$out_file"
        return 0
    fi
    {
        echo
        echo "################################################################"
        echo "# DIRECTORY: $label"
        echo "# PATH: $root"
        echo "################################################################"
        echo
    } >> "$out_file"
    find "$root" \
      -path "$root/target" -prune -o \
      -path "$root/.git" -prune -o \
      -path "$root/info" -prune -o \
      -type f \( \
        -name '*.rs' -o \
        -name '*.toml' -o \
        -name 'Cargo.toml' -o \
        -name 'Cargo.lock' -o \
        -name '*.md' -o \
        -name '*.sh' -o \
        -name '*.html' -o \
        -name '*.css' -o \
        -name '*.js' -o \
        -name '*.ts' -o \
        -name '*.json' -o \
        -name '*.ron' -o \
        -name '*.txt' -o \
        -name 'Containerfile' -o \
        -name 'Dockerfile' \
      \) -print \
    | sort \
    | while read -r file; do
        {
            echo
            echo "////////////////////////////////////////////////////////////////"
            echo "// FILE: $file"
            echo "////////////////////////////////////////////////////////////////"
            echo
            cat "$file"
            echo
        } >> "$out_file"
    done
}

append_crate_rs() {
    local crate_dir="$1"
    if [[ ! -d "$crate_dir/src" ]]; then
        return 0
    fi
    local crate_name
    crate_name="${crate_dir#./}"
    crate_name="${crate_name//\//_}"
    local crate_out="$OUT_DIR/src_${crate_name}_rs.txt"
    : > "$crate_out"
    {
        echo "================================================================"
        echo "RUST CRATE: $crate_name"
        echo "PATH: $crate_dir"
        echo "================================================================"
        echo
    } >> "$crate_out"
    find "$crate_dir/src" -type f -name '*.rs' -print \
    | sort \
    | while read -r rs_file; do
        {
            echo
            echo "////////////////////////////////////////////////////////////////"
            echo "// FILE: $rs_file"
            echo "////////////////////////////////////////////////////////////////"
            echo
            cat "$rs_file"
            echo
        } >> "$crate_out"
    done
    {
        echo
        echo "################################################################"
        echo "# RUST CRATE: $crate_name"
        echo "# PATH: $crate_dir"
        echo "################################################################"
        echo
        cat "$crate_out"
        echo
    } >> "$ALL_RS_OUT"
    echo "wrote $crate_out"
}

echo "== collecting trade root files =="
append_file "$ALL_OUT" "./Cargo.toml" "WORKSPACE CARGO TOML"
append_file "$ALL_OUT" "./Containerfile" "CONTAINERFILE"
append_file "$ALL_OUT" "./build_for_alpine.sh" "BUILD SCRIPT"
append_file "$ALL_OUT" "./.gitignore" "GITIGNORE"
append_file "$ALL_TOML_OUT" "./Cargo.toml" "WORKSPACE CARGO TOML"

echo "== collecting selected directories into aggregate =="
for dir in ./dsta ./ttrs ./usta ./vsta ./wsta ./wsta_makepad; do
    append_tree_files "$ALL_OUT" "$dir" "$dir"
done

echo "== collecting Rust source files =="
for crate_dir in ./dsta ./ttrs ./usta ./vsta ./wsta ./wsta_makepad; do
    append_crate_rs "$crate_dir"
done

echo "== collecting Cargo.toml files =="
find ./dsta ./ttrs ./usta ./vsta ./wsta ./wsta_makepad \
  -path '*/target' -prune -o \
  -path '*/.git' -prune -o \
  -name Cargo.toml -type f -print \
| sort \
| while read -r toml_file; do
    append_file "$ALL_TOML_OUT" "$toml_file" "CARGO TOML"
done

echo
echo "DONE."
echo "Main aggregate: $ALL_OUT"
echo "Rust aggregate: $ALL_RS_OUT"
echo "TOML aggregate: $ALL_TOML_OUT"
