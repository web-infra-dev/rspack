export default {
  "*.rs": "rustfmt --edition 2021",
  "*.{ts,js}": "pnpm run format:js",
  "*.toml": "npx @taplo/cli format",
  "*.{ts,js,mjs}": () => "pnpm run lint:js"
}
