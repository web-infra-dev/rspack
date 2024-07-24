import fs from "fs-extra";

import type { ECompilerType, TCompilerOptions } from "../type";

export function readConfigFile<T extends ECompilerType>(
	files: string[]
): TCompilerOptions<T>[] {
	const existsFile = files.find(i => fs.existsSync(i));
	const fileConfig = existsFile ? require(existsFile) : {};
	return Array.isArray(fileConfig) ? fileConfig : [fileConfig];
}
