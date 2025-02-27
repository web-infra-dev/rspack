import {
	type ECompilerType,
	EDocumentType,
	type ITestContext,
	type ITestEnv,
	type ITestRunner,
	type TCompilerOptions,
	type TCompilerStatsCompilation,
	type TRunnerFactory
} from "../type";
import { EsmRunner } from "./runner/esm";
import { WebRunner } from "./runner/web";

export class BasicRunnerFactory<T extends ECompilerType>
	implements TRunnerFactory<T>
{
	constructor(
		protected name: string,
		protected context: ITestContext
	) {}

	protected createStatsGetter() {
		const compiler = this.context.getCompiler<T>(this.name);
		const statsGetter = (() => {
			let cached: TCompilerStatsCompilation<T> | null = null;
			return () => {
				if (cached) {
					return cached;
				}
				cached = compiler.getStats()!.toJson({
					errorDetails: true
				});
				return cached;
			};
		})();
		return statsGetter;
	}

	create(
		file: string,
		compilerOptions: TCompilerOptions<T>,
		env: ITestEnv
	): ITestRunner {
		const key = this.getRunnerKey(file);
		const exists = this.context.getRunner(key);
		if (exists) {
			return exists;
		}
		const runner = this.createRunner(
			file,
			this.createStatsGetter(),
			compilerOptions,
			env
		);
		this.context.setRunner(key, runner);
		return runner;
	}

	protected getRunnerKey(file: string) {
		return this.name;
	}

	protected createRunner(
		file: string,
		stats: () => TCompilerStatsCompilation<T>,
		compilerOptions: TCompilerOptions<T>,
		env: ITestEnv
	): ITestRunner {
		const runnerOptions = {
			env,
			stats,
			name: this.name,
			testConfig: this.context.getTestConfig(),
			source: this.context.getSource(),
			dist: this.context.getDist(),
			compilerOptions
		};
		if (
			compilerOptions.target === "web" ||
			compilerOptions.target === "webworker"
		) {
			return new WebRunner<T>({
				...runnerOptions,
				runInNewContext: true,
				cachable: true,
				dom:
					this.context.getValue(this.name, "documentType") || EDocumentType.Fake
			});
		}
		return new EsmRunner<T>(runnerOptions);
	}
}
