import path from "path";
import postcssLoader from "@rspack/plugin-postcss";
import { test } from "uvu";
import assert from "assert";
import sinon from "sinon";
import { Rspack } from "../src";
import { Plugin } from "../src/config";
async function run(plugins: Plugin[]) {
	const rspack = new Rspack({
		entry: {
			main: path.resolve(__dirname, "../../../examples/postcss/index.js")
		},
		context: path.resolve(__dirname, "../../../examples/postcss"),
		plugins,
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
	const stats = await rspack.build();
	return stats;
}

test("basic", async () => {
	const donecallback = sinon.fake();
	const processAssetsCallback = sinon.fake();
	const stats = await run([
		{
			name: "done1",
			done() {
				console.log("done");
				donecallback();
			}
		},
		{
			name: "done1",
			done() {
				donecallback();
			}
		},
		{
			name: "process_assets1",
			processAssets(args) {
				processAssetsCallback(args);
			}
		}
	]);
	assert.equal(donecallback.callCount, 2);
	const keys = Object.keys(processAssetsCallback.args[0][0]).sort();
	assert.deepEqual(keys, ["main.css", "main.js", "runtime.js"]);
	assert.equal(stats.errors.length, 0);
});

test.run();
