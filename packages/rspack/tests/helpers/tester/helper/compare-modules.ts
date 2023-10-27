import { diffStringsUnified } from "jest-diff";
import { IFormatCodeOptions, formatCode } from "./format-code";
import { replaceRuntimeModuleName } from "./replace-runtime-module-name";

export const enum ECompareResultType {
	Same = "same",
	Missing = "missing",
	OnlyWebpack = "only-webpack",
	OnlyRspack = "only-rspack",
	Different = "different"
}
export type TCompareModules = string[] | true;
export type TCompareResult = {
	type: ECompareResultType;
	name: string;
	detail?: unknown;
};

export function compareModules(
	moduleList: TCompareModules,
	rspackModules: Map<string, string>,
	webpackModules: Map<string, string>,
	formatOptions: IFormatCodeOptions
) {
	const compareResults: TCompareResult[] = [];
	let compareModules: string[] = [];
	if (moduleList === true) {
		compareModules = [...rspackModules.keys(), ...webpackModules.keys()].filter(
			(i, idx, arr) => arr.indexOf(i) === idx
		);
	} else if (Array.isArray(moduleList)) {
		compareModules = moduleList;
	}
	for (let file of compareModules) {
		const renamed = replaceRuntimeModuleName(file);
		const rspackContent =
			rspackModules.has(renamed) &&
			formatCode(rspackModules.get(renamed)!, formatOptions);
		const webpackContent =
			webpackModules.has(renamed) &&
			formatCode(webpackModules.get(renamed)!, formatOptions);

		if (rspackContent) {
			if (webpackContent) {
				if (rspackContent === webpackContent) {
					compareResults.push({
						type: ECompareResultType.Same,
						name: file
					});
				} else {
					const difference = diffStringsUnified(
						rspackContent.trim(),
						webpackContent.trim()
					);
					compareResults.push({
						type: ECompareResultType.Different,
						name: file,
						detail: difference
					});
				}
			} else {
				compareResults.push({
					type: ECompareResultType.OnlyRspack,
					name: file
				});
			}
		} else {
			if (webpackContent) {
				compareResults.push({
					type: ECompareResultType.OnlyWebpack,
					name: file
				});
			} else {
				compareResults.push({
					type: ECompareResultType.Missing,
					name: file
				});
			}
		}
	}
	return compareResults;
}
