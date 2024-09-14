"use strict";

const HTMLContentForIndex = `
<!DOCTYPE html>
<html>
  <head>
    <meta
      http-equiv="Content-Security-Policy"
      content="require-trusted-types-for 'script'; trusted-types webpack webpack#dev-overlay;"
    />
    <meta charset='UTF-8'>
    <title>webpack-dev-server</title>
  </head>
  <body>
    <h1>webpack-dev-server is running...</h1>
    <script type="text/javascript" charset="utf-8" src="/main.js"></script>
  </body>
</html>
`;

const HTMLContentForTest = `
<!DOCTYPE html>
<html>
  <head>
    <meta
      http-equiv="Content-Security-Policy"
      content="require-trusted-types-for 'script'; trusted-types webpack webpack#dev-overlay;"
    />
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
			if (compiler.webpack) {
				const { RawSource } = compiler.webpack.sources;

				compilation.hooks.processAssets.tap(
					{
						name: pluginName,
						stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL
					},
					() => {
						const indexSource = new RawSource(HTMLContentForIndex);
						const testSource = new RawSource(HTMLContentForTest);

						compilation.emitAsset("index.html", indexSource);
						compilation.emitAsset("test.html", testSource);
					}
				);
			} else {
				compilation.hooks.additionalAssets.tap(pluginName, () => {
					compilation.emitAsset("index.html", {
						source() {
							return HTMLContentForIndex;
						},
						size() {
							return HTMLContentForIndex.length;
						}
					});
					compilation.emitAsset("test.html", {
						source() {
							return HTMLContentForTest;
						},
						size() {
							return HTMLContentForTest.length;
						}
					});
				});
			}
		});
	}
};
