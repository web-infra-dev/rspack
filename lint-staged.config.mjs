export default {
	"*.rs": "rustfmt --edition 2021",
	"*.{ts,js}": "pnpm run format:js",
	"*.toml": "npx @taplo/cli format",
	"*.{ts,js,cts,cjs,mts,mjs}": () => [
		"pnpm run lint:js",
		"pnpm run api-extractor:ci"
	]
};
