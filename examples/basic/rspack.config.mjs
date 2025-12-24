import { defineConfig } from "@rspack/cli";
import path from "node:path";

export default defineConfig({
	context: import.meta.dirname,
	entry: {
		index: "./src/index.js"
	},
	output: {
		path: path.resolve(import.meta.dirname, "dist")
	}
});
