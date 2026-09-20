import fs from "node:fs";
import path from "node:path";

export const pitch = function () {
	const request = this.utils.contextify(
		this.context,
		`${this.resourcePath}.css!=!-!${path.resolve(
			import.meta.dirname,
			"charset-style-loader.mjs"
		)}!${this.resourcePath}`
	);
	const source = fs.readFileSync(this.resourcePath, "utf-8");

	return `@import ${JSON.stringify(request)};${source}`;
};
