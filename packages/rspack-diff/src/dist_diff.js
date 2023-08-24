const fs = require("fs");
const os = require("os");
const path = require("path");
const { parseBundle } = require("./parseUtils");
const { dir } = require("console");
const { exec, execSync } = require("child_process");
/**
 *
 * @param {string} src
 * @param {string} dst
 */
function distDiffBundler(src, dst) {
	const srcResult = parseBundle(src);
	const dstResult = parseBundle(dst);
	/**
	 * diff modules
	 */
	const module1 = srcResult.modules;
	const module2 = dstResult.modules;
	console.log(Object.keys(module1));
	console.log(Object.keys(module2));
	const tmp = os.tmpdir();
	const dirctory = fs.mkdtempSync(tmp);
	for (const [key, value] of Object.entries(module1)) {
		fs.mkdirSync(path.resolve(dirctory, "src"), { recursive: true });
		fs.mkdirSync(path.resolve(dirctory, "dst"), { recursive: true });
		const srcTmp = path.resolve(dirctory, "src", key);
		const dstTmp = path.resolve(dirctory, "dst", key);

		fs.writeFileSync(srcTmp, module1[key], "utf-8");
		if (!module2[key]) {
			console.info(`${dst} missing module: ${key}`);
		} else {
			fs.writeFileSync(dstTmp, module2[key], "utf-8");
			const rsult = execSync(`difft ${srcTmp} ${dstTmp}`, { stdio: "inherit" });
		}
	}
	for (const key of Object.keys(module2)) {
		if (!module1[key]) {
			console.info(`${src} missing module: ${key}`);
		}
	}
}

module.exports = {
	distDiffBundler
};
