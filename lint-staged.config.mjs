export default {
  "*.rs": "rustfmt --edition 2021",
  "packages/**/*.{ts,js}": "prettier --write",
  "x.mjs": "prettier --write",
  "crates/rspack_plugin_runtime/**/*.{ts,js}": "prettier --write",
  "*.toml": "npx @taplo/cli format",
  "**/*.{ts,js,mjs}": () => "oxlint ."
}
