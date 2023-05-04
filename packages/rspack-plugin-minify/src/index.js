const isJsFile = /\.[cm]?js(?:\?.*)?$/i;
const esbuild = require("esbuild");
const terser = require("terser");
const { RawSource, SourceMapSource } = require("webpack-sources");
module.exports = class RspackMinifyPlugin {
	/**
	 *
	 * @param {{minifier: 'esbuild' | 'terser'}} options
	 */
	constructor(options) {
		this.options = options || {
			minifier: "esbuild",
			target: "es6"
		};
	}
	async transform(code, { sourcemap, sourcefile }) {
		if (this.options.minifier === "esbuild") {
			return await esbuild.transform(code, {
				target: this.options.target,
				sourcefile,
				sourcemap,
				minify: true,
				minifyIdentifiers: true,
				minifySyntax: true,
				minifyWhitespace: true
			});
		} else if (this.options.minifier === "terser") {
			const result = await terser.minify(
				{
					[sourcefile]: code
				},
				{
					sourceMap: sourcemap,
					ecma: this.options.target,
					mangle: this.options.mangle,
					keep_classnames: this.options.keep_classnames,
					keep_fnames: this.options.keep_fnames,
					compress: {
						passes: 2
					}
				}
			);
			return result;
		}
	}
	apply(compiler) {
		compiler.hooks.thisCompilation.tap("RspackMinifyPlugin", compilation => {
			compilation.hooks.processAssets.tapPromise(
				{
					name: "RspackMinifyPlugin"
				},
				async _ => {
					const {
						options: { devtool }
					} = compilation.compiler;
					const sourcemap = !!devtool;
					const assets = compilation.getAssets().filter(asset => {
						return isJsFile.test(asset.name);
					});

					await Promise.all(
						assets.map(async asset => {
							const { source, map } = asset.source.sourceAndMap();
							const sourceAsString = source.toString();
							const result = await this.transform(sourceAsString, {
								sourcemap,
								sourcefile: asset.name
							});
							compilation.updateAsset(
								asset.name,
								sourcemap
									? new SourceMapSource(
											result.code,
											asset.name,
											result.map,
											sourceAsString,
											map,
											true
									  )
									: new RawSource(result.code),
								{
									...asset.info,
									minimized: true
								}
							);
						})
					);
				}
			);
		});
	}
};
