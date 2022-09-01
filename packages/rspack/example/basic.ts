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
			name: "done1",
			done() {
				console.log("done1");
			}
		},
		{
			name: "done1",
			done() {
				console.log("done2");
			}
		},
		{
			name: "process_assets1",
			process_assets() {
				console.log("process_asssets1");
			}
		},
		{
			name: "process_assets2",
			process_assets(args) {
				for (const value of Object.values(args)) {
					//console.log("source:", value.source);
				}
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
