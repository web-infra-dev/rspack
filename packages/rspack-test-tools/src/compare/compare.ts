import fs from "fs-extra";
import { diffLinesRaw, diffStringsUnified } from "jest-diff";

import path from "node:path";
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

declare global {
	var updateSnapshot: boolean;
}

const WORKSPACE = path.resolve(__dirname, "../../../..");

export interface ICompareOptions {
	modules?: TCompareModules;
	runtimeModules?: TCompareModules;
	format: IFormatCodeOptions;
	renameModule?: (name: string) => string;
	bootstrap?: boolean;
	detail?: boolean;
	snapshot?: string;
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
	const distExists = compareOptions.snapshot
		? fs.existsSync(compareOptions.snapshot)
		: fs.existsSync(distFile);
	if (!sourceExists && !distExists) {
		result.type = ECompareResultType.Missing;
		return result;
	}
	if (!sourceExists && distExists) {
		result.type = ECompareResultType.OnlyDist;
		return result;
	}
	if (sourceExists && !distExists) {
		result.type = ECompareResultType.OnlySource;
		return result;
	}

	function formatModules(modules: Record<string, string>) {
		const res: Record<string, string> = {};
		for (const [name, content] of Object.entries(modules)) {
			const renamed = name.replaceAll(path.win32.sep, path.posix.sep);
			if (!renamed.includes("node_modules/css-loader/dist")) {
				res[renamed] = formatCode(renamed, content, compareOptions.format);
			}
		}
		return res;
	}

	const sourceContent = replaceRuntimeModuleName(
		fs.readFileSync(sourceFile, "utf-8").replaceAll(WORKSPACE, "__WORKSPACE__")
	);
	const sourceModules = parseModules(sourceContent, {
		bootstrap: compareOptions.bootstrap,
		renameModule: compareOptions.renameModule
	});
	sourceModules.modules = formatModules(sourceModules.modules);
	sourceModules.runtimeModules = formatModules(sourceModules.runtimeModules);

	let distModules = {
		modules: {},
		runtimeModules: {}
	};
	if (
		!global.updateSnapshot &&
		compareOptions.snapshot &&
		fs.existsSync(compareOptions.snapshot)
	) {
		distModules = JSON.parse(fs.readFileSync(compareOptions.snapshot, "utf-8"));
	} else {
		const distContent = replaceRuntimeModuleName(
			fs.readFileSync(distFile, "utf-8").replaceAll(WORKSPACE, "__WORKSPACE__")
		);
		distModules = parseModules(distContent, {
			bootstrap: compareOptions.bootstrap,
			renameModule: compareOptions.renameModule
		});
		distModules.modules = formatModules(distModules.modules);
		distModules.runtimeModules = formatModules(distModules.runtimeModules);

		if (compareOptions.snapshot) {
			fs.ensureDirSync(path.dirname(compareOptions.snapshot));
			fs.writeFileSync(
				compareOptions.snapshot,
				JSON.stringify(distModules, null, 2)
			);
		}
	}

	result.type = ECompareResultType.Different;

	for (const type of ["modules", "runtimeModules"]) {
		const t = type as "modules" | "runtimeModules";
		let moduleList: string[] = [];
		if (compareOptions[t] === true) {
			moduleList = [
				...Object.keys(sourceModules[t]),
				...Object.keys(distModules[t])
			].filter((i, idx, arr) => arr.indexOf(i) === idx);
		} else if (Array.isArray(compareOptions[t])) {
			moduleList = compareOptions[t] as string[];
		} else {
			continue;
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
	sourceModules: Record<string, string>,
	distModules: Record<string, string>,
	compareOptions: ICompareOptions
) {
	const compareResults: TModuleCompareResult[] = [];
	for (const name of modules) {
		const renamed = replaceRuntimeModuleName(name).replaceAll(
			path.win32.sep,
			path.posix.sep
		);
		const sourceContent = sourceModules[renamed];
		const distContent = distModules[renamed];

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
			}
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
	}
	return {
		type: ECompareResultType.Missing,
		lines: {
			source: 0,
			common: 0,
			dist: 0
		}
	};
}
