export default  
	{
    "*.rs": "rustfmt --edition 2021",
    "*.{ts,tsx,js}": "pnpm run format:js",
    "*.toml": "npx @taplo/cli format",
    "*.{ts,tsx,js,cts,cjs,mts,mjs}": [
      "pnpm run lint:js",
      // ignore staged file list; execute x without extra args
      ()=>'pnpm run x ae ci'
    ],
    "package.json": "pnpm run check-dependency-version"
  }
