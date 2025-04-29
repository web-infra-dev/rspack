const { rspack } = require("@rspack/core");

/** @type { import('@rspack/core').RspackOptions } */

module.exports = {
	context: __dirname,
	entry: {
		a: "./src/a.js",
		b: "./src/b.js",
	},
	output: {
		hotUpdateGlobal: "webpackHotUpdate_[runtime]",
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			// No `chunks` or `excludeChunks` added, we will have `<script src="a.js"/><script src="b.js"/>` in html
			// and the hotUpdateGlobal of b.js will override a.js' if they use the same name.
			templateContent: `<html><body><div id="a">aaa</div><div id="b">bbb</div></body></html>`,
		})
	],
	experiments: {
		css: true,
	}
};
