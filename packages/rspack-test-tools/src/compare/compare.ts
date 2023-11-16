import { diffLinesRaw, diffStringsUnified } from "jest-diff";
import { IFormatCodeOptions, formatCode } from "./format-code";
import { replaceRuntimeModuleName } from "./replace-runtime-module-name";
import { parseModules } from "../helper";
import fs from "fs-extra";
import {
	ECompareResultType,
	TCompareModules,
	TModuleCompareResult
} from "../type";

export interface ICompareOptions {
	modules?: TCompareModules;
	runtimeModules?: TCompareModules;
	format: IFormatCodeOptions;
	renameModule?: (name: string) => string;
}

export function compareFile(
	sourceFile: string,
	distFile: string,
	compareOptions: ICompareOptions
) {
	const sourceModules = parseModules(
		replaceRuntimeModuleName(fs.readFileSync(sourceFile, "utf-8"))
	);
	const distModules = parseModules(fs.readFileSync(distFile, "utf-8"));
	const result: Partial<
		Record<"modules" | "runtimeModules", TModuleCompareResult[]>
	> = {};
	for (let type of ["modules", "runtimeModules"]) {
		const t = type as "modules" | "runtimeModules";
		let moduleList: string[] = [];
		if (compareOptions[t] === true) {
			moduleList = [
				...sourceModules[t].keys(),
				...distModules[t].keys()
			].filter((i, idx, arr) => arr.indexOf(i) === idx);
		} else if (Array.isArray(compareOptions[t])) {
			moduleList = compareOptions[t] as string[];
		} else {
			continue;
		}
		if (typeof compareOptions.renameModule === "function") {
			moduleList = moduleList.map(compareOptions.renameModule);
		}
		result[t] = compareModules(
			moduleList,
			sourceModules[t],
			distModules[t],
			compareOptions.format
		);
	}
	return result;
}

export function compareModules(
	modules: string[],
	sourceModules: Map<string, string>,
	distModules: Map<string, string>,
	formatOptions: IFormatCodeOptions
) {
	const compareResults: TModuleCompareResult[] = [];
	for (let file of modules) {
		const renamed = replaceRuntimeModuleName(file);
		const sourceContent =
			sourceModules.has(renamed) &&
			formatCode(sourceModules.get(renamed)!, formatOptions);
		const distContent =
			distModules.has(renamed) &&
			formatCode(distModules.get(renamed)!, formatOptions);

		compareResults.push(compareContent(file, sourceContent, distContent));
	}
	return compareResults;
}

export function compareContent(
	name: string,
	sourceContent: string | false,
	distContent: string | false
): TModuleCompareResult {
	if (sourceContent) {
		if (distContent) {
			if (sourceContent === distContent) {
				return {
					type: ECompareResultType.Same,
					name
				};
			} else {
				const difference = diffStringsUnified(
					sourceContent.trim(),
					distContent.trim()
				);
				const lines = diffLinesRaw(
					sourceContent.trim().split("\n"),
					distContent.trim().split("\n")
				);
				return {
					type: ECompareResultType.Different,
					name,
					detail: difference,
					lines: {
						source: lines.filter(l => l[0] < 0).length,
						common: lines.filter(l => l[0] === 0).length,
						dist: lines.filter(l => l[0] > 0).length
					}
				};
			}
		} else {
			return {
				type: ECompareResultType.OnlySource,
				name
			};
		}
	} else {
		if (distContent) {
			return {
				type: ECompareResultType.OnlyDist,
				name
			};
		} else {
			return {
				type: ECompareResultType.Missing,
				name
			};
		}
	}
}
