import fs from "fs-extra";

import type { ECompilerType, ITestContext, TCompilerOptions } from "../type";

export function readConfigFile<T extends ECompilerType>(
	files: string[],
	context: ITestContext,
	prevOption?: TCompilerOptions<T>,
	functionApply?: (
		config: (
			| TCompilerOptions<T>
			| ((...args: unknown[]) => TCompilerOptions<T>)
		)[]
	) => TCompilerOptions<T>[]
): TCompilerOptions<T>[] {
	const existsFile = files.find(i => fs.existsSync(i));
	let fileConfig = existsFile ? require(existsFile) : {};
	if (typeof fileConfig === "function") {
		fileConfig = fileConfig(
			{ config: prevOption },
			{ testPath: context.getDist(), tempPath: context.getTemp() }
		);
	}
	const configArr = Array.isArray(fileConfig) ? fileConfig : [fileConfig];
	return functionApply ? functionApply(configArr) : configArr;
}
