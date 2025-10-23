import type { RspackOptions } from "@rspack/core";
import fs from "fs-extra";
import type { ITestContext } from "../type";

export function readConfigFile(
	files: string[],
	context: ITestContext,
	prevOption?: RspackOptions,
	functionApply?: (
		config: (RspackOptions | ((...args: unknown[]) => RspackOptions))[]
	) => RspackOptions[]
): RspackOptions[] {
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
