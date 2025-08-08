import { defineConfig } from "@rslib/core";

export default defineConfig({
	lib: [
		{ format: "cjs", syntax: ["node 18.12"], dts: { bundle: false } },
		{ format: "esm", syntax: ["node 18.12"] }
	],
	source: {
		tsconfigPath: "./tsconfig.build.json"
	}
});
