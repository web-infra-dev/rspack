#!/bin/bash

# Crates that need [lints.rust.unexpected_cfgs] replaced with [lints] workspace = true
REPLACE_CRATES=(
  "rspack_cacheable"
  "rspack_collections"
  "rspack_loader_swc"
  "rspack_plugin_html"
  "rspack_plugin_javascript"
  "rspack_plugin_mf"
  "rspack_plugin_real_content_hash"
  "rspack_plugin_rsc"
  "rspack_plugin_rsdoctor"
  "rspack_plugin_runtime"
  "rspack_plugin_sri"
  "rspack_util"
)

# Process all Cargo.toml files in crates/
for file in crates/*/Cargo.toml; do
  crate_name=$(basename $(dirname "$file"))

  # Check if this is one of the crates that needs replacement
  if [[ " ${REPLACE_CRATES[@]} " =~ " ${crate_name} " ]]; then
    echo "Replacing [lints.rust.unexpected_cfgs] in $file"
    # Remove the [lints.rust] section and add [lints] workspace = true
    sed -i '' '/^\[lints\.rust\]/,/^$/d' "$file"
    echo -e "\n[lints]\nworkspace = true" >> "$file"
  elif ! grep -q "^\[lints\]" "$file"; then
    # Only add if [lints] section doesn't exist
    echo "Adding [lints] workspace = true to $file"
    echo -e "\n[lints]\nworkspace = true" >> "$file"
  else
    echo "Skipping $file (already has [lints])"
  fi
done

echo "Done!"
