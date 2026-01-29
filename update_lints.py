#!/usr/bin/env python3

import os
import re
from pathlib import Path

# Crates that need [lints.rust.unexpected_cfgs] replaced
REPLACE_CRATES = {
    "rspack_cacheable",
    "rspack_collections",
    "rspack_loader_swc",
    "rspack_plugin_html",
    "rspack_plugin_javascript",
    "rspack_plugin_mf",
    "rspack_plugin_real_content_hash",
    "rspack_plugin_rsc",
    "rspack_plugin_rsdoctor",
    "rspack_plugin_runtime",
    "rspack_plugin_sri",
    "rspack_util",
}

def update_cargo_toml(file_path):
    crate_name = file_path.parent.name

    with open(file_path, 'r') as f:
        content = f.read()

    # Check if already has [lints] workspace = true
    if re.search(r'^\[lints\]\s*\nworkspace\s*=\s*true', content, re.MULTILINE):
        print(f"✓ {crate_name}: Already has workspace lints")
        return False

    # Check if this is one of the crates that needs replacement
    if crate_name in REPLACE_CRATES:
        # Remove [lints.rust] section (including all its content until next section or EOF)
        pattern = r'\n\[lints\.rust\.unexpected_cfgs\].*?(?=\n\[|\Z)'
        new_content = re.sub(pattern, '', content, flags=re.DOTALL)

        # Add [lints] workspace = true
        if not new_content.endswith('\n'):
            new_content += '\n'
        new_content += '\n[lints]\nworkspace = true\n'

        with open(file_path, 'w') as f:
            f.write(new_content)

        print(f"✓ {crate_name}: Replaced [lints.rust.unexpected_cfgs] with workspace lints")
        return True

    # Check if has any [lints] section
    elif re.search(r'^\[lints', content, re.MULTILINE):
        print(f"! {crate_name}: Has custom [lints] section (skipping)")
        return False

    # Add [lints] workspace = true to files without any [lints] section
    else:
        if not content.endswith('\n'):
            content += '\n'
        content += '\n[lints]\nworkspace = true\n'

        with open(file_path, 'w') as f:
            f.write(content)

        print(f"✓ {crate_name}: Added workspace lints")
        return True

def main():
    crates_dir = Path("crates")
    updated_count = 0

    for cargo_toml in sorted(crates_dir.glob("*/Cargo.toml")):
        if update_cargo_toml(cargo_toml):
            updated_count += 1

    print(f"\nUpdated {updated_count} Cargo.toml files")

if __name__ == "__main__":
    main()
