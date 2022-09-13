import path from "path";
import postcssLoader from "@rspack/plugin-postcss";
import { test } from "uvu";
import assert from "assert";
import sinon from "sinon";
import { Rspack, Plugin } from "../src";
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
			name: "test-plugin",
			apply(compiler) {
				compiler.hooks.done.tap("done1", () => {
					console.log("done");
					donecallback();
				});
				compiler.hooks.done.tap("done2", () => {
					donecallback();
				});
				compiler.hooks.compilation.tap("compilation", compilation => {
					compilation.hooks.processAssets.tap("assets", assets => {
						processAssetsCallback(assets);
					});
				});
			}
		} as Plugin
	]);
	assert.equal(donecallback.callCount, 2);
	const keys = Object.keys(processAssetsCallback.args[0][0]).sort();
	assert.deepEqual(keys, ["main.css", "main.js", "runtime.js"]);
	assert.equal(stats.errors.length, 0);
});
test("sass", async () => {
	const context = path.resolve(__dirname, "../../../examples/react-with-sass");
	const rspack = new Rspack({
		entry: {
			main: path.resolve(context, "./src/index.jsx")
		},
		context,
		module: {
			rules: [
				{
					test: "\\.s[ac]ss$",
					uses: [{ builtinLoader: "sass-loader" }],
					type: "css"
				}
			]
		}
	});
	const stats = await rspack.build();
	assert(stats.errors.length == 0, "should contains no error");
});

test.run();
