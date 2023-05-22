/** @type {import('@rspack/cli').Configuration} */

// more info here: https://dev.to/mellis481/how-to-inspect-files-packaged-by-webpack-before-they-are-emitted-337j
const convertSourceToString = source => {
	if (typeof source === "string") {
		return source;
	} else {
		return new TextDecoder().decode(source);
	}
};

class AssetValidatorPlugin {
	apply(compiler) {
		compiler.hooks.shouldEmit.tap("AssetValidatorPlugin", compilation => {
			this.validateAssets(compilation);
		});
	}

	validateAssets(compilation) {
		const assets = Object.entries(compilation.assets);
		const regex = new RegExp("MY_SUPER_SECRET", "g");

		for (let i = 0; i < assets.length; i++) {
			const [fileName] = assets[i];
			const asset = compilation.getAsset(fileName);
			const source = asset.source.source();
			const contents = convertSourceToString(source);
			const matches = contents.match(regex);

			if (matches) {
				throw new Error(
					"Our tool has identified the presence of the string 'MY_SUPER_SECRET' in your compiled code. Compilation has been aborted."
				);
			}
		}

		return true;
	}
}

const config = {
	context: __dirname,
	entry: {
		main: "./src/index.js"
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		]
	},
	plugins: [new AssetValidatorPlugin()]
};
module.exports = config;
