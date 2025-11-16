// var MCEP = require("mini-css-extract-plugin");
var MCEP = require("@rspack/core").CssExtractRspackPlugin;

/** @type {(number, any) => import("@rspack/core").Configuration} */
const config = (i, options) => ({
	entry: {
		a: "./a",
		b: "./b",
		c: "./c.css",
		x: "./x" // also imports chunk but with different exports
	},
	output: {
		filename: `${i}_[name].js`,
	},
	experiments: {
		css: false
	},
	module: {
		rules: [
			{
				oneOf: [
					{
						test: /\.css$/,
						use: [MCEP.loader, "css-loader"]
					},
					{ test: /\.js$/ },
					{ type: "asset" }
				]
			}
		]
	},
	optimization: {
		chunkIds: "named"
	},
	target: "web",
	node: {
		__dirname: false
	},
	plugins: [
		new MCEP(options),
		compiler => {
			compiler.hooks.done.tap("Test", stats => {
				const chunkIds = stats
					.toJson({ all: false, chunks: true, ids: true })
					.chunks.map(c => c.id)
					.sort();

				// The two dynamic chunks have a stable prefix but non-stable suffixes across runtimes (native vs wasm).
				const dynamicChunkIds = chunkIds.filter(id => id.startsWith("chunk_js-_"));
				const staticChunkIds = chunkIds.filter(id => !id.startsWith("chunk_js-_")).sort();

				expect(dynamicChunkIds).toHaveLength(2);
				expect(staticChunkIds).toEqual([
					"a",
					"b",
					"c",
					"d_css",
					"x"
				]);
			});
		}
	]
});

module.exports = [
	config(0),
	config(1, {
		experimentalUseImportModule: true
	})
];
