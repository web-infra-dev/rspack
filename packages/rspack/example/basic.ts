import path from "path";
import { Rspack } from "../src";

const rspack = new Rspack({
	entry: {
		main: path.resolve(__dirname, "../../../examples/react/src/index.js")
	},
	context: path.resolve(__dirname, "../../../examples/react"),
	plugins: ["html"],
	module: {
		rules: [
			{
				test: ".less$",
				uses: [
					{
						loader: function testLoader(loaderContext) {
							return {
								content: loaderContext.source.getBuffer()
							};
						}
					},
					{
						loader: function testLoader2(loaderContext) {
							return {
								content: loaderContext.source.getBuffer()
							};
						}
					}
				],
				type: "css"
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
