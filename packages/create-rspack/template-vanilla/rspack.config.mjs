import { defineConfig } from "@rspack/cli";
import { rspack } from "@rspack/core";

export default defineConfig({
	entry: {
		main: "./src/index.js"
	},
	plugins: [new rspack.HtmlRspackPlugin({ template: "./index.html" })]
});
