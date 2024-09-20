import { defineConfig } from "@rslib/core";

export default defineConfig({
	lib: [{ format: "esm" }],
	output: {
		target: "node"
	}
});
