import {
	type ECompilerType,
	EDocumentType,
	type ITestEnv,
	type ITestRunner,
	type TCompilerOptions,
	type TCompilerStatsCompilation
} from "../type";
import { BasicRunnerFactory } from "./basic";
import { WebRunner } from "./runner";

export class WatchRunnerFactory<
	T extends ECompilerType
> extends BasicRunnerFactory<T> {
	protected getRunnerKey(file: string): string {
		const stepName: string | void = this.context.getValue(
			this.name,
			"watchStepName"
		);
		return `${this.name}-${stepName}`;
	}

	protected createStatsGetter() {
		const compiler = this.context.getCompiler<T>(this.name);
		const stepName: string = this.context.getValue(this.name, "watchStepName")!;
		const statsGetter = (() => {
			const cached: Record<string, TCompilerStatsCompilation<T>> = {};
			return () => {
				if (cached[stepName]) {
					return cached[stepName];
				}
				cached[stepName] = compiler.getStats()!.toJson({
					errorDetails: true
				});
				return cached[stepName];
			};
		})();
		return statsGetter;
	}

	protected createRunner(
		file: string,
		stats: () => TCompilerStatsCompilation<T>,
		compilerOptions: TCompilerOptions<T>,
		env: ITestEnv
	): ITestRunner {
		this.context.getCompiler<T>(this.name);
		const stepName: string | void = this.context.getValue(
			this.name,
			"watchStepName"
		);
		if (!stepName) {
			throw new Error("Can not get watch step name from context");
		}

		const state: Record<string, any> | void = this.context.getValue(
			this.name,
			"watchState"
		);
		if (!state) {
			throw new Error("Can not get watch state from context");
		}

		const isWeb = Array.isArray(compilerOptions)
			? compilerOptions.some(option => {
					return option.target === "web" || option.target === "webworker";
				})
			: compilerOptions.target === "web" ||
				compilerOptions.target === "webworker";

		const testConfig = this.context.getTestConfig();
		const documentType: EDocumentType =
			this.context.getValue(this.name, "documentType") || EDocumentType.Fake;
		return new WebRunner({
			dom: documentType,
			env,
			stats,
			name: this.name,
			runInNewContext: isWeb,
			cachable: false,
			testConfig: {
				...(testConfig || {}),
				moduleScope: (ms, stats, options) => {
					ms.STATE = state;
					ms.WATCH_STEP = stepName;
					if (typeof testConfig.moduleScope === "function") {
						return testConfig.moduleScope(ms, stats, options);
					}
					return ms;
				}
			},
			source: this.context.getSource(),
			dist: this.context.getDist(),
			compilerOptions
		});
	}
}
