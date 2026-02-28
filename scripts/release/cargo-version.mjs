import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import TOML from '@iarna/toml';

const __dirname = path.dirname(new URL(import.meta.url).pathname);

/**
 * Read the version from the workspace root Cargo.toml file
 * @returns {string} The version string from Cargo.toml
 * @throws {Error} If the Cargo.toml file cannot be read or parsed
 */
export function getCargoVersion() {
  try {
    const cargoTomlPath = resolve(__dirname, '..', '..', 'Cargo.toml');
    const cargoTomlContent = readFileSync(cargoTomlPath, 'utf8');
    const parsed = TOML.parse(cargoTomlContent);

    const version = parsed.workspace?.package?.version;

    if (!version) {
      throw new Error(
        'No version found in Cargo.toml workspace.package or package section',
      );
    }

    return version;
  } catch (error) {
    console.error('Error reading Cargo.toml:', error);
    throw error;
  }
}

/**
 * Create a tag name with the specified prefix and version
 * @param {string} version - The version string
 * @param {string} prefix - The prefix for the tag (default: "crates@")
 * @returns {string} The formatted tag name
 */
export function createTagName(version, prefix = 'crates@') {
  return `${prefix}${version}`;
}
