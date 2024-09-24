import fs from "fs-extra";

import type { ECompilerType, TCompilerOptions } from "../type";

export function readConfigFile<T extends ECompilerType>(
	files: string[],
	functionApply?: (
		config: (
			| TCompilerOptions<T>
			| ((...args: unknown[]) => TCompilerOptions<T>)
		)[]
	) => TCompilerOptions<T>[]
): TCompilerOptions<T>[] {
	const existsFile = files.find(i => fs.existsSync(i));
	const fileConfig = existsFile ? require(existsFile) : {};
	const configArr = Array.isArray(fileConfig) ? fileConfig : [fileConfig];
	return functionApply ? functionApply(configArr) : configArr;
}
