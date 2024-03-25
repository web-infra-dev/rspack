import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestRunner,
	TCompilerOptions,
	TRunnerFactory
} from "../type";
import { WebRunner } from "./runner/web";
import { EsmRunner } from "./runner/esm";

export class BasicRunnerFactory<T extends ECompilerType>
	implements TRunnerFactory<T>
{
	constructor(
		protected name: string,
		protected context: ITestContext
	) {}

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
		const runner = this.createRunner(file, compilerOptions, env);
		this.context.setRunner(key, runner);
		return runner;
	}

	protected getRunnerKey(file: string) {
		return this.name;
	}

	protected createRunner(
		file: string,
		compilerOptions: TCompilerOptions<T>,
		env: ITestEnv
	): ITestRunner {
		const compiler = this.context.getCompiler<T>(this.name);
		const stats = compiler.getStats();
		const runnerOptions = {
			env,
			stats: stats!,
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
			return new WebRunner<T>(runnerOptions);
		} else {
			return new EsmRunner<T>(runnerOptions);
		}
	}
}
