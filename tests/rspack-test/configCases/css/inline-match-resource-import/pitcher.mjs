import path from "node:path";

export const pitch = function () {
	const request = this.utils.contextify(
		this.context,
		`${this.resourcePath}.css!=!-!${path.resolve(
			import.meta.dirname,
			"style-loader.mjs"
		)}!${this.resourcePath}`
	);

	return `@import ${JSON.stringify(request)};`;
};
