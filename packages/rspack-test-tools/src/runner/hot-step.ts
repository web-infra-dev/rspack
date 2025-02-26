import type { StatsCompilation } from "@rspack/core";

import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import {
	type ECompilerType,
	EDocumentType,
	type ITestEnv,
	type ITestRunner,
	type TCompilerOptions,
	type TCompilerStats,
	type TCompilerStatsCompilation,
	type TUpdateOptions
} from "../type";
import { HotRunnerFactory } from "./hot";
import { WebRunner } from "./runner/web";
import type { THotStepRuntimeData } from "./type";

declare let global: {
	__CHANGED_FILES__: Map<string, number>;
};

export class HotStepRunnerFactory<
	T extends ECompilerType
> extends HotRunnerFactory<T> {
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
				const checker = this.context.getValue(
					this.name,
					jsonStats.errors?.length
						? "hotUpdateStepErrorChecker"
						: "hotUpdateStepChecker"
				) as (
					context: { updateIndex: number },
					stats: TCompilerStats<T>,
					runtime: THotStepRuntimeData
				) => void;
				checker(
					hotUpdateContext,
					stats as TCompilerStats<T>,
					runner.getGlobal("__HMR_UPDATED_RUNTIME__") as THotStepRuntimeData
				);

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

		const runner = new WebRunner({
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

		return runner;
	}
}
