export default {
	"*.rs": "rustfmt --edition 2021",
	"*.{ts,tsx,js}": "pnpm run format:js",
	"*.toml": "npx @taplo/cli format",
	"*.{ts,tsx,js,cts,cjs,mts,mjs}": () => [
		"pnpm run lint:js",
		"pnpm run lint:js-sort-imports-order",
		"pnpm run api-extractor:ci"
	]
};
