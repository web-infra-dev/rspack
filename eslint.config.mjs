import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import tsParser from "@typescript-eslint/parser";
import oxlint from "eslint-plugin-oxlint";
import simpleImportSort from "eslint-plugin-simple-import-sort";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ignores = readFileSync(path.resolve(__dirname, ".eslintignore"), "utf-8")
	.split(/\r?\n/)
	.filter(item => {
		return !item.startsWith("#") && !item.startsWith("//") && item.trim();
	});

export default [
	{
		files: ["**/*.ts", "**/*.mts", "**/*.cts"],
		languageOptions: {
			parser: tsParser
		}
	},
	{
		plugins: {
			"simple-import-sort": simpleImportSort
		},
		rules: {
			"simple-import-sort/imports": "error",
			"simple-import-sort/exports": "error"
		}
	},
	oxlint.configs["flat/recommended"], // oxlint should be the last one
	{
		ignores
	}
];
