"use strict";

const oneHTMLContent = `
<!doctype html>
<html>
  <head>
    <meta charset='UTF-8'>
    <title>test</title>
    <script src="one.js"></script>
  </head>
  <body></body>
</html>
`;
const twoHTMLContent = `
<!doctype html>
<html>
  <head>
    <meta charset='UTF-8'>
    <title>test</title>
    <script src="two.js"></script>
  </head>
  <body></body>
</html>
`;

module.exports = {
	devtool: false,
	mode: "development",
	context: __dirname,
	stats: "none",
	entry: {
		one: "./one.js",
		two: "./two.js"
	},
	output: {
		path: "/"
	},
	experiments: {
		lazyCompilation: true
	},
	infrastructureLogging: {
		level: "info",
		stream: {
			write: () => {}
		}
	},
	plugins: [
		{
			apply(compiler) {
				const pluginName = "html-generator-plugin-test";
				const oneFilename = "test-one.html";
				const twoFilename = "test-two.html";

				compiler.hooks.thisCompilation.tap(pluginName, compilation => {
					const { RawSource } = compiler.webpack.sources;

					compilation.hooks.processAssets.tap(
						{
							name: pluginName,
							stage:
								compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL
						},
						() => {
							const oneSource = new RawSource(oneHTMLContent);

							compilation.emitAsset(oneFilename, oneSource);

							const twoSource = new RawSource(twoHTMLContent);

							compilation.emitAsset(twoFilename, twoSource);
						}
					);
				});
			}
		}
	]
};
