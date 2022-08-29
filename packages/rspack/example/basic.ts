import path from "path";
import postcssLoader from "rspack-plugin-postcss";
import { Rspack } from "../src";

const rspack = new Rspack({
	entry: {
		main: path.resolve(__dirname, "../../../examples/postcss/index.js")
	},
	context: path.resolve(__dirname, "../../../examples/postcss"),
	plugins: [],
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
					// 			extraData: Buffer.from("something")
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
