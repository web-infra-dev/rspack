const readAsset = require("./readAsset");
function transformWindowPath(path) {
	if (process.platform === "win32") {
		return path.replace(/\\/g, "/");
	}
	return path;
}
function readAssets(compiler, stats) {
	const assets = {};

	Reflect.ownKeys(stats.compilation.assets)
		.filter(a => a !== "main.js")
		.forEach(asset => {
			assets[transformWindowPath(asset)] = readAsset(asset, compiler, stats);
		});

	return assets;
}

readAssets.transformWindowPath = transformWindowPath;
module.exports = readAssets;
