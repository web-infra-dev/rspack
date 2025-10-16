import { createRequire } from "node:module";
import { defineConfig } from "@rslib/core";

const require = createRequire(import.meta.url);

export default defineConfig({
	lib: [
		{ format: "cjs", syntax: ["node 18.12"], dts: { bundle: false } },
		{ format: "esm", syntax: ["node 18.12"] }
	],
	source: {
		tsconfigPath: "./tsconfig.build.json",
		define: {
			RSPACK_CLI_VERSION: JSON.stringify(require("./package.json").version)
		}
	}
});
