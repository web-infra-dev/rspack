const { createFsFromVolume, Volume } = require("memfs");
let statsJson;

class TestPlugin {
	apply(compiler) {
		compiler.hooks.thisCompilation.tap(TestPlugin.name, compilation => {
			compilation.hooks.processAssets.tapAsync(
				TestPlugin.name,
				async (assets, callback) => {
					const child = compiler.createChildCompiler(
						compilation,
						"TestChild",
						1,
						compilation.outputOptions,
						[
							new compiler.webpack.EntryPlugin(
								compiler.context,
								"./fixtures/abc",
								{ name: "TestChild" }
							)
						]
					);
					child.runAsChild(err => callback(err));
				}
			);
		});
		compiler.hooks.done.tap("test plugin", stats => {
			statsJson = stats.toJson({
				all: false,
				children: true,
				assets: true
			});
		});
	}
}

/** @type {import('../..').TStatsAPICaseConfig} */
module.exports = {
	description: "should have children when using childCompiler",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/a",
			plugins: [new TestPlugin()]
		};
	},
	async check(stats) {
		expect(statsJson).toMatchInlineSnapshot(`
		Object {
		  assets: Array [
		    Object {
		      auxiliaryChunkIdHints: Array [],
		      auxiliaryChunkNames: Array [],
		      cached: false,
		      chunkIdHints: Array [],
		      chunkNames: Array [],
		      emitted: true,
		      info: Object {
		        chunkhash: Array [],
		        contenthash: Array [],
		        fullhash: Array [],
		        isOverSizeLimit: false,
		        javascriptModule: false,
		        minimized: true,
		        related: Object {},
		      },
		      name: TestChild.js,
		      size: 344,
		      type: asset,
		    },
		    Object {
		      auxiliaryChunkIdHints: Array [],
		      auxiliaryChunkNames: Array [],
		      cached: false,
		      chunkIdHints: Array [],
		      chunkNames: Array [
		        main,
		      ],
		      emitted: true,
		      info: Object {
		        chunkhash: Array [],
		        contenthash: Array [],
		        fullhash: Array [],
		        isOverSizeLimit: false,
		        javascriptModule: false,
		        minimized: true,
		        related: Object {},
		      },
		      name: main.js,
		      size: 204,
		      type: asset,
		    },
		  ],
		  assetsByChunkName: Object {
		    main: Array [
		      main.js,
		    ],
		  },
		  children: Array [
		    Object {
		      assets: Array [
		        Object {
		          auxiliaryChunkIdHints: Array [],
		          auxiliaryChunkNames: Array [],
		          cached: false,
		          chunkIdHints: Array [],
		          chunkNames: Array [
		            TestChild,
		          ],
		          emitted: true,
		          info: Object {
		            chunkhash: Array [],
		            contenthash: Array [],
		            fullhash: Array [],
		            javascriptModule: false,
		            minimized: true,
		            related: Object {},
		          },
		          name: TestChild.js,
		          size: 344,
		          type: asset,
		        },
		      ],
		      assetsByChunkName: Object {
		        TestChild: Array [
		          TestChild.js,
		        ],
		      },
		      children: Array [],
		      filteredAssets: undefined,
		      name: TestChild,
		    },
		  ],
		  filteredAssets: undefined,
		}
	`);
	}
};
