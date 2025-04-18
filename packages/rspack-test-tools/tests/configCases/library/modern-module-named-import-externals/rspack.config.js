module.exports = {
	mode: "none",
	entry: { main: "./index.js", test: "./test.js" },
	output: {
		module: true,
		library: {
			type: "modern-module"
		},
		filename: "[name].js",
		chunkFormat: "module"
	},
	experiments: {
		outputModule: true
	},
	resolve: {
		extensions: [".js"]
	},
	externalsType: "module",
	externals: [
		"externals0",
		"externals1",
		"externals2",
		"externals3",
		"externals4"
	],
	optimization: {
		concatenateModules: true,
		usedExports: true
	},
	plugins: [
		function () {
			const handler = compilation => {
				compilation.hooks.processAssets.tap("testcase", assets => {
					const source = assets["test.js"].source();
					expect(source).toMatchInlineSnapshot(`
				import { a, b } from "externals0";
				import { a as a_0 } from "externals1";
				import default_0 from "externals2";
				import * as __WEBPACK_EXTERNAL_MODULE_externals3__ from "externals3";
				import "externals4";

				;// CONCATENATED MODULE: external "externals0"

				;// CONCATENATED MODULE: external "externals1"

				;// CONCATENATED MODULE: external "externals2"

				;// CONCATENATED MODULE: external "externals3"

				;// CONCATENATED MODULE: external "externals4"

				;// CONCATENATED MODULE: ./lib.js

				b;

				;// CONCATENATED MODULE: ./test.js
				// re export


				// named import
				;

				// default import


				// namespace import


				// side effect only import




				a_0;
				default_0;
				__WEBPACK_EXTERNAL_MODULE_externals3__;

				export { a };
			`);
				});
			};
			this.hooks.compilation.tap("testcase", handler);
		}
	]
};
