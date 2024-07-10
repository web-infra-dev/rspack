import fs from "fs-extra";
import { diffLinesRaw, diffStringsUnified } from "jest-diff";

import { parseModules } from "../helper";
import {
	ECompareResultType,
	type TCompareModules,
	type TCompareResult,
	type TFileCompareResult,
	type TModuleCompareResult
} from "../type";
import { type IFormatCodeOptions, formatCode } from "./format-code";
import { replaceRuntimeModuleName } from "./replace-runtime-module-name";

export interface ICompareOptions {
	modules?: TCompareModules;
	runtimeModules?: TCompareModules;
	format: IFormatCodeOptions;
	renameModule?: (name: string) => string;
	bootstrap?: boolean;
	detail?: boolean;
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

	const sourceModules = parseModules(sourceContent, {
		bootstrap: compareOptions.bootstrap
	});
	const distModules = parseModules(distContent, {
		bootstrap: compareOptions.bootstrap
	});

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
			compareOptions
		);
	}
	return result;
}

export function compareModules(
	modules: string[],
	sourceModules: Map<string, string>,
	distModules: Map<string, string>,
	compareOptions: ICompareOptions
) {
	const compareResults: TModuleCompareResult[] = [];
	for (let name of modules) {
		const renamed = replaceRuntimeModuleName(name);
		const sourceContent =
			sourceModules.has(renamed) &&
			formatCode(name, sourceModules.get(renamed)!, compareOptions.format);
		const distContent =
			distModules.has(renamed) &&
			formatCode(name, distModules.get(renamed)!, compareOptions.format);

		compareResults.push({
			...compareContent(sourceContent, distContent, compareOptions),
			name
		});
	}
	return compareResults;
}

export function compareContent(
	sourceContent: string | false,
	distContent: string | false,
	compareOptions: ICompareOptions
): TCompareResult {
	if (sourceContent) {
		if (distContent) {
			if (sourceContent === distContent) {
				const lines = sourceContent.trim().split("\n").length;
				return {
					type: ECompareResultType.Same,
					source: sourceContent,
					dist: distContent,
					lines: {
						source: 0,
						common: lines,
						dist: 0
					}
				};
			} else {
				const difference = compareOptions.detail
					? diffStringsUnified(sourceContent.trim(), distContent.trim())
					: undefined;
				const diffLines = diffLinesRaw(
					sourceContent.trim().split("\n"),
					distContent.trim().split("\n")
				);
				return {
					type: ECompareResultType.Different,
					detail: difference,
					source: sourceContent,
					dist: distContent,
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
				source: sourceContent,
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
				dist: distContent,
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
