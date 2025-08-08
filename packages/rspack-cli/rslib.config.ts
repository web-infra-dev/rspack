import { defineConfig } from "@rslib/core";

export default defineConfig({
	lib: [
		{ format: "cjs", syntax: ["node 18"], dts: { bundle: false } },
		{ format: "esm", syntax: ["node 18"] }
	],
	source: {
		tsconfigPath: "./tsconfig.build.json"
	}
});
