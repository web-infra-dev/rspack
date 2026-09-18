/** @type {import("@rspack/core").LoaderDefinition<{ i: string }>} */
export default function () {
	const options = this.getOptions();
	const i = +options.i;
	let src = `import n from "./async.js";\n`;
	if (i > 0) {
		src += `import a from "./loader.mjs?i=${i - 1}&a!./loader.mjs";\n`;
		src += `import b from "./loader.mjs?i=${i - 1}&b!./loader.mjs";\n`;
		src += `export default n + a + b;\n`;
	} else {
		src += `export default n;\n`;
	}
	return src;
};
