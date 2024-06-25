import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "@rspack/cli";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig([
	{
		mode: "production",
		entry: path.resolve(__dirname, "main.ts"),
		output: {
			path: path.resolve(__dirname, "dist"),
			filename: "ts.bundle.js"
		}
	}
]);
