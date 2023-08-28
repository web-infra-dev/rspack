const fs = require("fs");
const path = require("path");
/**
 *
 * @param {{rspack_stats:string, webpack_stats:string}} param0
 */
function statDiff({ rspack_stats, webpack_stats }) {
	console.log("xxx:", rspack_stats);
	const rspackStats = require(path.resolve(process.cwd(), rspack_stats));
	const webpackStats = require(path.resolve(process.cwd(), webpack_stats));

	const rspackModules = new Set(rspackStats.modules.map(x => x.identifier));
	const webpackModules = new Set(webpackStats.modules.map(x => x.identifier));
	for (const module of rspackModules) {
		if (!webpackModules.has(module)) {
			console.info(`webpack missing ${module}`);
		}
	}
	for (const module of webpackModules) {
		if (!rspackModules.has(module)) {
			console.info(`rspack missing ${module}`);
		}
	}
}

module.exports = {
	statDiff
};
