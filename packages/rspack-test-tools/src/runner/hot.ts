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

declare let global: {
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

		const next = async (
			callback?: (
				error: Error | null,
				stats?: TCompilerStatsCompilation<T>
			) => void
		) => {
			const usePromise = typeof callback === "function";
			try {
				hotUpdateContext.updateIndex++;
				const stats = await compiler.build();
				if (!stats) {
					throw new Error("Should generate stats during build");
				}
				const jsonStats = stats.toJson({
					// errorDetails: true
				});

				await checkArrayExpectation(
					source,
					jsonStats,
					"error",
					`errors${hotUpdateContext.updateIndex}`,
					"Error"
				);
				await checkArrayExpectation(
					source,
					jsonStats,
					"warning",
					`warnings${hotUpdateContext.updateIndex}`,
					"Warning"
				);
				if (usePromise) {
					// old callback style hmr cases
					callback(null, jsonStats as StatsCompilation);
				} else {
					// new promise style hmr cases
					return jsonStats as StatsCompilation;
				}
			} catch (e) {
				if (usePromise) {
					callback(e as Error);
				} else {
					throw e;
				}
			}
		};

		const nextHMR = async (m: any, options?: any) => {
			const jsonStats = await next();
			const updatedModules = await m.hot.check(options || true);
			if (!updatedModules) {
				throw new Error("No update available");
			}
			return jsonStats as StatsCompilation;
		};

		return new WebRunner({
			dom:
				this.context.getValue(this.name, "documentType") || EDocumentType.JSDOM,
			env,
			stats: this.createStatsGetter(),
			name: this.name,
			runInNewContext: false,
			testConfig: {
				...testConfig,
				moduleScope(ms, stats) {
					const moduleScope =
						typeof testConfig.moduleScope === "function"
							? testConfig.moduleScope(ms, stats)
							: ms;

					moduleScope.NEXT = next;
					moduleScope.NEXT_HMR = nextHMR;
					return moduleScope;
				}
			},
			cachable: true,
			source,
			dist,
			compilerOptions
		});
	}
}
