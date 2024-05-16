const isJsFile = /\.[cm]?js(?:\?.*)?$/i;
const isCssFile = /\.css(?:\?.*)?$/i;

const esbuild = require("esbuild");
const { minify } = require("terser");
const { RawSource, SourceMapSource } = require("webpack-sources");
module.exports = class RspackMinifyPlugin {
	/**
	 *
	 * @param {{minifier: 'esbuild' | 'terser'}} options
	 */
	constructor(options) {
		this.options = {
			minifier: "esbuild",
			target: "es6",
			css: false,
			...options
		};
	}
	async transform(code, { sourcemap, sourcefile, css }) {
		if (this.options.minifier === "esbuild" || css) {
			return await esbuild.transform(code, {
				loader: css ? "css" : "js",
				target: this.options.target,
				sourcefile,
				sourcemap,
				format: "iife",
				minify: true,
				minifyIdentifiers: true,
				minifySyntax: true,
				minifyWhitespace: true
			});
		} else if (this.options.minifier === "terser") {
			const options = Object.assign({}, this.options);
			delete options.minifier;
			delete options.target;
			delete options.css;
			const result = await minify(
				{
					[sourcefile]: code
				},
				{
					sourceMap: sourcemap,
					...options
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
					const assets = compilation.getAssets().filter(asset => {
						return (
							// Don't double minimize assets
							!asset.info.minimized &&
							(isJsFile.test(asset.name) ||
								(this.options.css && isCssFile.test(asset.name)))
						);
					});

					await Promise.all(
						assets.map(async asset => {
							const { source, map } = asset.source.sourceAndMap();
							const sourceAsString = source.toString();
							const result = await this.transform(sourceAsString, {
								sourcemap: !!map,
								css: isCssFile.test(asset.name),
								sourcefile: asset.name
							});
							compilation.updateAsset(
								asset.name,
								map
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
