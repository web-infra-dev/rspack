const path = require("path");

class Plugin {
	apply(compiler) {
		const moduleMap = new Map();

		compiler.hooks.compilation.tap("PLUGIN", compilation => {
			compilation.hooks.finishModules.tap("PLUGIN", modules => {
				for (const module of modules) {
					if (moduleMap.has(module.resource)) {
						const timestamp = moduleMap.get(module.resource);
						// index.js only run loader by once.
						expect(module.buildInfo.timestamp).toBe(timestamp);
					} else {
						moduleMap.set(module.resource, module.buildInfo.timestamp);
					}
				}
			});
		});
	}
}

module.exports = {
	plugins: [new Plugin()],
	module: {
		rules: [
			{
				test: /\.js$/,
				use: [
					{
						loader: path.join(__dirname, "loader.js")
					}
				]
			}
		]
	}
};
