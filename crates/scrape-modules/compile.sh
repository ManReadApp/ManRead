#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"


declare -a folder_names
for dir in "$SCRIPT_DIR"/*/; do
  if [ -d "$dir" ] && [ "$(basename "$dir")" != "target" ]; then
    folder_name=$(basename "$dir")
    folder_names+=("$folder_name")
    if [ -f "$dir/Cargo.toml" ]; then
      echo "Compiling project in $dir"
      cargo build --release --manifest-path "$dir/Cargo.toml"
    fi
  fi
done

  if [ -n "$CARGO_TARGET_DIR" ]; then
    TARGET_DIR="$CARGO_TARGET_DIR"
  else
    TARGET_DIR="$SCRIPT_DIR/target"
  fi

find "$TARGET_DIR/release" -type f \( -name "*.so" -o -name "*.dylib" -o -name "*.dll" \) -print0 | while IFS= read -r -d $'\0' file; do
  filename=$(basename "$file")

  if [[ ! "$filename" == lib* ]]; then
    continue
  fi

  base_name="${filename#lib}"
  extension="${filename##*.}"
  base_name="${base_name%.*}"

  found_match=false
  for folder in "${folder_names[@]}"; do
    if [[ "$base_name" == "$folder" ]]; then
      found_match=true
      break
    fi
  done

  if [[ "$found_match" == false ]]; then
    echo "Skipping $filename - doesn't match any folder name"
    continue
  fi

  new_filename="${base_name}.${extension}"

  echo "Moving $filename to $SCRIPT_DIR/target/binaries/$new_filename"
  mv "$file" "data/binaries/$new_filename"
done
