export default [
	// each time sets different assetsInfo object instance in rspack.config.mjs
	// this prevents hit in inmemory cache
	/^Pack got invalid because of write to: TerserWebpackPlugin|bundle0\.js$/
];
