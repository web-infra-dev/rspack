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

				expect(chunkIds).toEqual(process.env.WASM ? [
					"a",
					"b",
					"c",
					"chunk_js-_d5940",
					"chunk_js-_d5941",
					"d_css",
					"x"
				] : [
					"a",
					"b",
					"c",
					"chunk_js-_aaff0",
					"chunk_js-_aaff1",
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
