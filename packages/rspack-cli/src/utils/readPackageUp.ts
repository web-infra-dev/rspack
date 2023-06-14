import fs from "fs";
import path from "path";
import { PackageJson } from "type-fest";

const readPackageUp = (cwd = process.cwd()): PackageJson | null => {
	let currentDir = path.resolve(cwd);
	let packageJsonPath = path.join(currentDir, "package.json");

	while (!fs.existsSync(packageJsonPath)) {
		let parentDir = path.dirname(currentDir);
		if (parentDir === currentDir) {
			return null;
		}
		currentDir = parentDir;
		packageJsonPath = path.join(currentDir, "package.json");
	}
	try {
		return JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
	} catch (error) {
		return null;
	}
};

export default readPackageUp;
