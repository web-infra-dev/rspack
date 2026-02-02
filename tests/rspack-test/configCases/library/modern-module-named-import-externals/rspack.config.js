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
						import { HomeLayout as external_externals0_HomeLayout, a } from "externals0";
						import { a as external_externals1_a } from "externals1";
						import externals2 from "externals2";
						import * as __rspack_external_externals3 from "externals3";
						import "externals4";









						(function Layout(props) {
						  const { HomeLayout = external_externals0_HomeLayout } = props;
						  call({ HomeLayout });
						})()

						// re export


						// named import
						;

						// default import


						// namespace import


						// side effect only import




						external_externals1_a;
						externals2;
						__rspack_external_externals3;

						export { a };
					`);
				});
			};
			this.hooks.compilation.tap("testcase", handler);
		}
	]
};
