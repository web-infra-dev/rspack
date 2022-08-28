import path from "path";
import { Rspack } from "../src";

const rspack = new Rspack({
	entry: {
		main: path.resolve(__dirname, "../../../examples/test/index.js")
	},
	context: path.resolve(__dirname, "../../../examples/test"),
	plugins: [
	],
	module: {
		rules: [
			{
				test: ".module.css$",
				uses: [
					{
						loader: function testLoader(loaderContext) {
							// console.log(loaderContext);
							return {
								content: loaderContext.source.getBuffer(),
								metaData: Buffer.from("something")
							};
						}
					}
				]
			},
		]
	}
});

async function main() {
	const stats = await rspack.build();
	console.log(stats);
	// assert(stats.assets.length > 0)
}

main();
