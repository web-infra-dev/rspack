import path from "path";
import postcssLoader from "@rspack/plugin-postcss";
import { Rspack } from "../src";

const rspack = new Rspack({
	entry: {
		main: path.resolve(__dirname, "../../../examples/postcss/index.js")
	},
	context: path.resolve(__dirname, "../../../examples/postcss"),
	plugins: [
		{
			name: "test",
			apply(compiler) {
				compiler.hooks.done.tap("done1", () => {
					console.log("done1");
				});
				compiler.hooks.done.tap("done2", () => {
					console.log("done2");
				});
				compiler.hooks.processAssets.tapPromise(
					"processAssets1",
					async (...args) => {
						console.log({ args });
					}
				);
				compiler.hooks.processAssets.tap("processAssets2", args => {
					console.log({ args });
				});
			}
		}
	],
	module: {
		rules: [
			{
				test: ".module.css$",
				uses: [
					{
						loader: postcssLoader,
						options: {
							modules: true
						}
					}
					// {
					// 	loader: function testLoader(loaderContext) {
					// 		// console.log(loaderContext);
					// 		return {
					// 			content: loaderContext.source.getBufFer(),
					// 			meta: Buffer.from("something")
					// 		};
					// 	}
					// }
				]
			}
		]
	}
});

async function main() {
	const stats = await rspack.build();
	console.log(stats);
	// assert(stats.assets.length > 0)
}

main();
