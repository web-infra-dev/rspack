import type { StatsCompilation } from "@rspack/core";

import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import {
	type ECompilerType,
	EDocumentType,
	type ITestEnv,
	type ITestRunner,
	type TCompilerOptions,
	type TCompilerStatsCompilation,
	type TUpdateOptions
} from "../type";
import { BasicRunnerFactory } from "./basic";
import { WebRunner } from "./runner/web";

declare var global: {
	__CHANGED_FILES__: Map<string, number>;
};

export class HotRunnerFactory<
	T extends ECompilerType
> extends BasicRunnerFactory<T> {
	protected createRunner(
		file: string,
		stats: TCompilerStatsCompilation<T>,
		compilerOptions: TCompilerOptions<T>,
		env: ITestEnv
	): ITestRunner {
		const compiler = this.context.getCompiler(this.name);
		const testConfig = this.context.getTestConfig();
		const source = this.context.getSource();
		const dist = this.context.getDist();
		const hotUpdateContext = this.context.getValue<TUpdateOptions>(
			this.name,
			"hotUpdateContext"
		)!;

		const next = (
			callback: (
				error: Error | null,
				stats?: TCompilerStatsCompilation<T>
			) => void
		) => {
			hotUpdateContext.updateIndex++;
			// TODO: find a better way to collect changed files from fake-update-loader
			const changedFiles = new Map();
			global["__CHANGED_FILES__"] = changedFiles;
			compiler
				.build()
				.then(stats => {
					if (!stats)
						return callback(new Error("Should generate stats during build"));
					const jsonStats = stats.toJson({
						// errorDetails: true
					});

					hotUpdateContext.totalUpdates = Math.max(
						hotUpdateContext.totalUpdates,
						...changedFiles.values()
					);
					hotUpdateContext.changedFiles = [...changedFiles.keys()];

					if (
						checkArrayExpectation(
							source,
							jsonStats,
							"error",
							"errors" + hotUpdateContext.updateIndex,
							"Error",
							callback
						)
					) {
						return;
					}
					if (
						checkArrayExpectation(
							source,
							jsonStats,
							"warning",
							"warnings" + hotUpdateContext.updateIndex,
							"Warning",
							callback
						)
					) {
						return;
					}
					callback(null, jsonStats as StatsCompilation);
				})
				.catch(callback);
		};

		return new WebRunner({
			dom:
				this.context.getValue(this.name, "documentType") || EDocumentType.JSDOM,
			env,
			stats,
			name: this.name,
			runInNewContext: false,
			testConfig: {
				...testConfig,
				moduleScope(ms) {
					if (typeof testConfig.moduleScope === "function") {
						ms = testConfig.moduleScope(ms);
					}
					ms["NEXT"] = next;
					return ms;
				}
			},
			source,
			dist,
			compilerOptions
		});
	}
}
