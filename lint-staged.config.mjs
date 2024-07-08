export default {
	"*.rs": "rustfmt --edition 2021",
	"*.{ts,tsx,js}": "pnpm run format:js",
	"*.toml": "npx @taplo/cli format",
	"*.{ts,tsx,js,cts,cjs,mts,mjs}": () => ["pnpm run lint:js", "node ./x ae ci"]
};
