import { diffLinesRaw, diffStringsUnified } from "jest-diff";
import { IFormatCodeOptions, formatCode } from "./format-code";
import { replaceRuntimeModuleName } from "./replace-runtime-module-name";
import { parseModules } from "../helper";
import fs from "fs-extra";
import {
	ECompareResultType,
	TCompareModules,
	TCompareResult,
	TFileCompareResult,
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
): TFileCompareResult {
	const result: TFileCompareResult = {
		type: ECompareResultType.Same,
		file: {
			source: sourceFile,
			dist: distFile
		},
		modules: {}
	};
	const sourceExists = fs.existsSync(sourceFile);
	const distExists = fs.existsSync(distFile);
	if (!sourceExists && !distExists) {
		result.type = ECompareResultType.Missing;
		return result;
	} else if (!sourceExists && distExists) {
		result.type = ECompareResultType.OnlyDist;
		return result;
	} else if (sourceExists && !distExists) {
		result.type = ECompareResultType.OnlySource;
		return result;
	}

	const sourceContent = replaceRuntimeModuleName(
		fs.readFileSync(sourceFile, "utf-8")
	);
	const distContent = replaceRuntimeModuleName(
		fs.readFileSync(distFile, "utf-8")
	);

	// const compareContentResult = compareContent(sourceContent, distContent);
	// result.detail = compareContentResult.detail;
	// result.lines = compareContentResult.lines;
	result.type = ECompareResultType.Different;

	const sourceModules = parseModules(sourceContent);
	const distModules = parseModules(distContent);

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
		result.modules[t] = compareModules(
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
	for (let name of modules) {
		const renamed = replaceRuntimeModuleName(name);
		const sourceContent =
			sourceModules.has(renamed) &&
			formatCode(sourceModules.get(renamed)!, formatOptions);
		const distContent =
			distModules.has(renamed) &&
			formatCode(distModules.get(renamed)!, formatOptions);

		compareResults.push({
			...compareContent(sourceContent, distContent),
			name
		});
	}
	return compareResults;
}

export function compareContent(
	sourceContent: string | false,
	distContent: string | false
): TCompareResult {
	if (sourceContent) {
		if (distContent) {
			if (sourceContent === distContent) {
				const lines = sourceContent.trim().split("\n").length;
				return {
					type: ECompareResultType.Same,
					lines: {
						source: lines,
						common: lines,
						dist: lines
					}
				};
			} else {
				const difference = diffStringsUnified(
					sourceContent.trim(),
					distContent.trim()
				);
				const diffLines = diffLinesRaw(
					sourceContent.trim().split("\n"),
					distContent.trim().split("\n")
				);
				return {
					type: ECompareResultType.Different,
					detail: difference,
					lines: {
						source: diffLines.filter(l => l[0] < 0).length,
						common: diffLines.filter(l => l[0] === 0).length,
						dist: diffLines.filter(l => l[0] > 0).length
					}
				};
			}
		} else {
			return {
				type: ECompareResultType.OnlySource,
				lines: {
					source: sourceContent.trim().split("\n").length,
					common: 0,
					dist: 0
				}
			};
		}
	} else {
		if (distContent) {
			return {
				type: ECompareResultType.OnlyDist,
				lines: {
					source: 0,
					common: 0,
					dist: distContent.trim().split("\n").length
				}
			};
		} else {
			return {
				type: ECompareResultType.Missing,
				lines: {
					source: 0,
					common: 0,
					dist: 0
				}
			};
		}
	}
}
