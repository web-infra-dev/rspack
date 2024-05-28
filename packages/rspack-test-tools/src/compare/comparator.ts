import deepmerge from "deepmerge";
import path from "path";

import { ITestReporter, TCompareModules } from "../type";
import { compareFile } from "./compare";
import { IFormatCodeOptions } from "./format-code";
import { replaceRuntimeModuleName } from "./replace-runtime-module-name";

export interface IDiffComparatorOptions {
	rspackDist: string;
	webpackDist: string;
	files: string[];
	modules?: TCompareModules;
	runtimeModules?: TCompareModules;
	reporters: ITestReporter<unknown>[];
	formatOptions?: IFormatCodeOptions;
	bootstrap?: boolean;
}

export class DiffComparator {
	constructor(private options: IDiffComparatorOptions) {}
	async compare() {
		for (let file of this.options.files!) {
			try {
				const result = compareFile(
					path.join(this.options.rspackDist, file),
					path.join(this.options.webpackDist, file),
					{
						modules: this.options.modules,
						runtimeModules: this.options.runtimeModules,
						format: deepmerge(
							{
								replacements: {},
								ignorePropertyQuotationMark: true,
								ignoreModuleId: true,
								ignoreModuleArguments: true,
								ignoreBlockOnlyStatement: true,
								ignoreIfCertainCondition: true,
								ignoreSwcHelpersPath: true,
								ignoreObjectPropertySequence: true,
								ignoreCssFilePath: true
							},
							this.options.formatOptions || {}
						),
						renameModule: replaceRuntimeModuleName,
						bootstrap: this.options.bootstrap
					}
				);
				for (let reporter of this.options.reporters) {
					reporter.increment(file, result.modules["modules"] || []);
				}
				for (let reporter of this.options.reporters) {
					reporter.increment(file, result.modules["runtimeModules"] || []);
				}
			} catch (e) {
				console.error(e);
				for (let reporter of this.options.reporters) {
					reporter.failure(file);
				}
			}
		}
		await Promise.all(this.options.reporters.map(r => r.output()));
	}
}
