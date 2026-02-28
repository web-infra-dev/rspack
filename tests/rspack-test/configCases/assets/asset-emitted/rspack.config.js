"use strict";

const Source = require("@rspack/core").sources.Source;
const Compilation = require("@rspack/core").Compilation;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		compiler => {
			/** @type {Record<string, boolean>} */
			const files = {};
			compiler.hooks.assetEmitted.tap(
				"Test",
				(file, { content, source, outputPath, compilation, targetPath }) => {
					expect(Buffer.isBuffer(content)).toBe(true);
					expect(source).toBeInstanceOf(Source);
					expect(typeof outputPath).toBe("string");
					expect(typeof targetPath).toBe("string");
					expect(compilation).toBeInstanceOf(Compilation);
					files[file] = true;
				}
			);
			compiler.hooks.afterEmit.tap("Test", () => {
				expect(files).toMatchInlineSnapshot(`
			Object {
			  694.bundle0.js: true,
			  bundle0.js: true,
			}
		`);
			});
		}
	]
};
