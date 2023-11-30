import path from "path";
import fs from "fs";

// TODO: It's better to use `Promise.all`
function configFileExist(dir: string, config: "webpack" | "rspack") {
	const files = ["js", "ts", "mjs", "mts"].map(
		ext => `${config}.config.${ext}`
	);
	return files.some(file => {
		const p = path.resolve(dir, file);
		return fs.existsSync(p) && fs.lstatSync(p).isFile();
	});
}

export function ensureWebpackConfigExist(testCaseDir: string) {
	const exist = configFileExist(testCaseDir, "webpack");
	if (!exist) {
		throw Error(`not found webpack.config.js in ${testCaseDir}`);
	}
}

export function ensureRspackConfigNotExist(testCaseDir: string) {
	const exist = configFileExist(testCaseDir, "rspack");
	if (exist) {
		throw Error(`rspack config file should not exist in ${testCaseDir}`);
	}
}
