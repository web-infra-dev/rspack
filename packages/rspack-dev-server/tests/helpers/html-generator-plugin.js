"use strict";

const HTMLContentForIndex = `
<!DOCTYPE html>
<html>
  <head>
    <meta charset='UTF-8'>
    <title>webpack-dev-server</title>
  </head>
  <body>
    <h1>webpack-dev-server is running...</h1>
    <script type="text/javascript" charset="utf-8" src="/main.js"></script>
  </body>
</html>
`;

const HTMLContentForAssets = assetName => `
<!DOCTYPE html>
<html>
  <head>
    <meta charset='UTF-8'>
    <title>webpack-dev-server</title>
  </head>
  <body>
    <h1>(${assetName}>)webpack-dev-server is running...</h1>
    <script type="text/javascript" charset="utf-8" src=${assetName}></script>
  </body>
</html>
`;

const HTMLContentForTest = `
<!DOCTYPE html>
<html>
  <head>
    <meta charset='UTF-8'>
    <title>test</title>
  </head>
  <body>
    <h1>Created via HTMLGeneratorPlugin</h1>
  </body>
</html>
`;

module.exports = class HTMLGeneratorPlugin {
	// eslint-disable-next-line class-methods-use-this
	apply(compiler) {
		const pluginName = "html-generator-plugin";

		compiler.hooks.thisCompilation.tap(pluginName, compilation => {
			const { RawSource } = compiler.webpack.sources;

			compilation.hooks.processAssets.tap(
				{
					name: pluginName,
					stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL
				},
				() => {
					const indexSource = new RawSource(HTMLContentForIndex);
					const testSource = new RawSource(HTMLContentForTest);
					const assets = compilation.getAssets();

					compilation.emitAsset("index.html", indexSource);
					compilation.emitAsset("test.html", testSource);

					for (const asset of assets) {
						const assetName = asset.name;

						if (assetName !== "main.js") {
							const assetSource = new RawSource(
								HTMLContentForAssets(assetName)
							);
							compilation.emitAsset(
								assetName.replace(".js", ".html"),
								assetSource
							);
						}
					}
				}
			);
		});
	}
};
