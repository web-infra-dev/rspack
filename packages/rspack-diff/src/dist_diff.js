const fs = require("fs");
const os = require("os");
const path = require("path");
const { parseBundle } = require("./parseUtils");
const { dir } = require("console");
const { exec, execSync } = require("child_process");
const { result } = require("lodash");
/**
 *
 * @param {string} src
 * @param {string} dst
 */
function distDiffBundler(rspack_dist, webpack_dist) {
	const srcResult = parseBundle(rspack_dist);
	const dstResult = parseBundle(webpack_dist);
	/**
	 * diff modules
	 */
	const module1 = srcResult.modules;
	const module2 = dstResult.modules;
	console.log(Object.keys(module1));
	console.log(Object.keys(module2));
	const dirctory = path.resolve(process.cwd(), ".diff");
	fs.mkdirSync(dirctory, { recursive: true });
	let i = 0;
	for (const [key, value] of Object.entries(module1)) {
		fs.mkdirSync(path.resolve(dirctory, "rspack"), { recursive: true });
		fs.mkdirSync(path.resolve(dirctory, "webpack"), { recursive: true });
		const srcTmp = path.resolve(dirctory, "rspack", path.basename(key));
		const dstTmp = path.resolve(dirctory, "webpack", path.basename(key));
		fs.writeFileSync(srcTmp, module1[key], "utf-8");
		if (!module2[key]) {
			console.info(`webpack missing module: ${key}`);
		} else {
			fs.writeFileSync(dstTmp, module2[key], "utf-8");
			const result = execSync(`difft ${srcTmp} ${dstTmp}`, {
				stdio: "inherit"
			});
		}
	}
	for (const key of Object.keys(module2)) {
		if (!module1[key]) {
			console.info(`rspack missing module: ${key}`);
		}
	}
}

module.exports = {
	distDiffBundler
};
