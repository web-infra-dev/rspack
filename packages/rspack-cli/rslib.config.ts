import { defineConfig } from "@rslib/core";

export default defineConfig({
	lib: [
		{ format: "cjs", syntax: "es2021", dts: { bundle: false } },
		{ format: "esm", syntax: "es2021" }
	],
	source: {
		tsconfigPath: "./tsconfig.build.json"
	}
});
