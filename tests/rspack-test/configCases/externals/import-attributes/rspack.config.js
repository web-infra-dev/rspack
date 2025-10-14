const path = require("path");
const fs = require("fs");
const {
	Compilation,
	sources: { RawSource }
} = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: {
			type: "module"
		}
	},
	target: ["web", "es2020"],
	experiments: {
		outputModule: true
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.compilation.tap("html-plugin", compilation => {
					compilation.hooks.processAssets.tap(
						{
							name: "copy-plugin",
							stage: Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL
						},
						() => {
							[
								"static-package.json",
								"static-package-str.json",
								"dynamic-package.json",
								"dynamic-package-str.json",
								"eager.json",
								"weak.json",
								"./nested/pkg.json",
								"re-export.json",
								"re-export-directly.json"
							].forEach(filename => {
								const resolvedFilename = path.resolve(__dirname, filename);
								const content = fs.readFileSync(resolvedFilename);
								compilation.emitAsset(
									filename.replace(/\.\/nested\//, ""),
									new RawSource(content)
								);
							});

							const content = compilation.getAsset("bundle0.mjs").source.source()
							const esmImportSpecifier1 = content.match(/import (.+) from "\.\/static-package\.json" with \{"type":"json"\};/);
							const esmImportSpecifier2 = content.match(/import (.+) from "\.\/static-package\.json";/);
							expect(esmImportSpecifier1[1]).not.toBe(esmImportSpecifier2[1]);
							const importChunkId1 = content.match(/const dynamicPkgPure = await __webpack_require__.e\(\/\* import\(\) \*\/ "(.+)"\)/)
							const importChunkId2 = content.match(/const dynamicPkgStr = await __webpack_require__.e\(\/\* import\(\) \*\/ "(.+)"\)/)
							expect(importChunkId1[1]).not.toBe(importChunkId2[1]);
						}
					);
				});
			}
		}
	],
	externals: {
		"./static-package.json": "module ./static-package.json",
		"./static-package-str.json": "module ./static-package-str.json",
		"./dynamic-package.json": "import ./dynamic-package.json",
		"./dynamic-package-str.json": "import ./dynamic-package-str.json",
		"./eager.json": "import ./eager.json",
		"./weak.json": "import ./weak.json",
		"./pkg.json": "import ./pkg.json",
		"./pkg": "import ./pkg.json",
		"./re-export.json": "module ./re-export.json",
		"./re-export-directly.json": "module ./re-export-directly.json"
	}
};
