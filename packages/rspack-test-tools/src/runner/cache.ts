import type { StatsCompilation } from "@rspack/core";

import type { HotUpdatePlugin } from "../helper/hot-update";
import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import {
	type ECompilerType,
	EDocumentType,
	type ITestEnv,
	type ITestRunner,
	type TCompilerOptions,
	type TCompilerStatsCompilation
} from "../type";
import { BasicRunnerFactory } from "./basic";
import { WebRunner } from "./runner/web";

const MAX_COMPILER_INDEX = 100;

export class CacheRunnerFactory<
	T extends ECompilerType
> extends BasicRunnerFactory<T> {
	protected createRunner(
		file: string,
		stats: TCompilerStatsCompilation<T>,
		compilerOptions: TCompilerOptions<T>,
		env: ITestEnv
	): ITestRunner {
		const compiler = this.context.getCompiler(this.name);
		let compilerIndex = 0;
		const testConfig = this.context.getTestConfig();
		const source = this.context.getSource();
		const dist = this.context.getDist();
		const updatePlugin = this.context.getValue<HotUpdatePlugin>(
			this.name,
			"hotUpdateContext"
		)!;
		const getWebRunner = () => {
			return new WebRunner({
				dom:
					this.context.getValue(this.name, "documentType") ||
					EDocumentType.JSDOM,
				env,
				stats: this.createStatsGetter(),
				cachable: false,
				name: this.name,
				runInNewContext: false,
				testConfig: {
					...testConfig,
					moduleScope(ms, stats) {
						const moduleScope =
							typeof testConfig.moduleScope === "function"
								? testConfig.moduleScope(ms, stats, compilerOptions)
								: ms;

						moduleScope.COMPILER_INDEX = compilerIndex;
						moduleScope.NEXT_HMR = nextHmr;
						moduleScope.NEXT_START = nextStart;
						return moduleScope;
					}
				},
				source,
				dist,
				compilerOptions
			});
		};
		const nextHmr = async (
			m: any,
			options?: any
		): Promise<TCompilerStatsCompilation<T>> => {
			await updatePlugin.goNext();
			const stats = await compiler.build();
			if (!stats) {
				throw new Error("Should generate stats during build");
			}
			const jsonStats = stats.toJson({
				// errorDetails: true
			});
			const compilerOptions = compiler.getOptions();

			const updateIndex = updatePlugin.getUpdateIndex();
			await checkArrayExpectation(
				source,
				jsonStats,
				"error",
				`errors${updateIndex}`,
				"Error",
				compilerOptions
			);
			await checkArrayExpectation(
				source,
				jsonStats,
				"warning",
				`warnings${updateIndex}`,
				"Warning",
				compilerOptions
			);

			const updatedModules = await m.hot.check(options || true);
			if (!updatedModules) {
				throw new Error("No update available");
			}

			return jsonStats as StatsCompilation;
		};

		const nextStart = async (): Promise<TCompilerStatsCompilation<T>> => {
			await compiler.close();
			compiler.createCompiler();
			await updatePlugin.goNext();
			const stats = await compiler.build();
			if (!stats) {
				throw new Error("Should generate stats during build");
			}
			const jsonStats = stats.toJson({
				// errorDetails: true
			});
			const compilerOptions = compiler.getOptions();

			const updateIndex = updatePlugin.getUpdateIndex();
			await checkArrayExpectation(
				source,
				jsonStats,
				"error",
				`errors${updateIndex}`,
				"Error",
				compilerOptions
			);
			await checkArrayExpectation(
				source,
				jsonStats,
				"warning",
				`warnings${updateIndex}`,
				"Warning",
				compilerOptions
			);
			env.it(
				`NEXT_START run with compilerIndex==${compilerIndex + 1}`,
				async () => {
					if (compilerIndex > MAX_COMPILER_INDEX) {
						throw new Error(
							"NEXT_START has been called more than the maximum times"
						);
					}
					compilerIndex++;
					return getWebRunner().run(file);
				}
			);
			return jsonStats as StatsCompilation;
		};

		return getWebRunner();
	}
}
