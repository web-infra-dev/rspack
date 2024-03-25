const path = require("path");
const fs = require("fs");

/**
 * Ensures that the rspack/webpack config file exists in the specified directory.
 * Throws an error if the config file does not exist.
 *
 * @param {string} dir - The directory to check for the config file.
 * @param {"webpack" | "webpack"} config - Config type to check.
 */
// TODO: It's better to use `Promise.all`
function configFileExist(dir, config) {
	const files = ["js", "ts", "mjs", "mts"].map(
		ext => `${config}.config.${ext}`
	);
	return files.some(file => {
		const p = path.resolve(dir, file);
		return fs.existsSync(p) && fs.lstatSync(p).isFile();
	});
}

/**
 * Ensures that the webpack config file exists in the specified directory.
 * Throws an error if the config file does not exist.
 *
 * @param {string} testCaseDir - The directory to check for the config file.
 */
function ensureWebpackConfigExist(testCaseDir) {
	const exist = configFileExist(testCaseDir, "webpack");
	if (!exist) {
		throw Error(`not found webpack.config.js in ${testCaseDir}`);
	}
}

/**
 * Ensures that the rspack config file does not exist in the specified directory.
 * Throws an error if the config file exists.
 *
 * @param {string} testCaseDir - The directory to check for the config file.
 */
function ensureRspackConfigNotExist(testCaseDir) {
	const exist = configFileExist(testCaseDir, "rspack");
	if (exist) {
		throw Error(`rspack config file should not exist in ${testCaseDir}`);
	}
}

/**
 * Ensures that the rspack config file does not exist in the specified directory.
 * Throws an error if the config file exists.
 *
 * @param {string} testCaseDir - The directory to check for the config file.
 */
function isValidTestCaseDir(testCaseDir) {
	return !testCaseDir.startsWith("_") && !testCaseDir.startsWith(".");
}

module.exports = {
	isValidTestCaseDir,
	ensureRspackConfigNotExist,
	ensureWebpackConfigExist
};
