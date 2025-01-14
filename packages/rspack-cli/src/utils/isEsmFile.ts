import path from "node:path";

import readPackageUp from "./readPackageUp";

const isEsmFile = (filePath: string, cwd = process.cwd()) => {
	const ext = path.extname(filePath);
	if (/\.(mjs|mts)$/.test(ext)) {
		return true;
	}
	if (/\.(cjs|cts)/.test(ext)) {
		return false;
	}
	// package.json should be find from configPath root
	const packageJson = readPackageUp(path.dirname(filePath));
	return packageJson?.type === "module";
};

export default isEsmFile;
