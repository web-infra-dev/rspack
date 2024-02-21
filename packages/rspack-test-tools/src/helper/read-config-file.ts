import path from "path";
import { ECompilerType, TCompilerOptions } from "../type";
import fs from "fs-extra";

export function readConfigFile<T extends ECompilerType>(
	root: string,
	files: string[]
): TCompilerOptions<T>[] {
	const existsFile = files
		.map(i => path.resolve(root, i))
		.find(i => fs.existsSync(i));
	const fileConfig = existsFile ? require(existsFile) : {};
	return Array.isArray(fileConfig) ? fileConfig : [fileConfig];
}
