import path from "path";
import { ECompilerType, TCompilerOptions } from "../type";
import fs from "fs-extra";
import deepmerge from "deepmerge";

export function readConfigFile<T extends ECompilerType>(
	root: string,
	files: string[],
	options: TCompilerOptions<T>
): TCompilerOptions<T> {
	const existsFile = files
		.map(i => path.resolve(root, i))
		.find(i => fs.existsSync(i));
	const fileConfig: TCompilerOptions<T> = existsFile ? require(existsFile) : {};
	return deepmerge<TCompilerOptions<T>>(options, fileConfig);
}
