import path from "node:path";

import { TestCompilerManager } from "../compiler";
import type {
	ITestCompilerManager,
	ITestContext,
	ITestEnv,
	ITesterConfig,
	ITestRunner,
	TTestConfig
} from "../type";

export type TTestContextOptions = Omit<ITesterConfig, "steps">;

export class TestContext implements ITestContext {
	protected errors: Error[] = [];
	protected compiler: ITestCompilerManager | null = null;
	protected store: Map<string, unknown> = new Map();
	protected runners: Map<string, ITestRunner> = new Map();

	constructor(private config: TTestContextOptions) {}

	getSource(sub?: string): string {
		if (sub) {
			return path.resolve(this.config.src, sub);
		}
		return this.config.src;
	}

	getDist(sub?: string): string {
		if (sub) {
			return path.resolve(this.config.dist, sub);
		}
		return this.config.dist;
	}

	getTemp(sub?: string): string | null {
		if (!this.config.temp) return null;
		if (sub) {
			return path.resolve(this.config.temp, sub);
		}
		return this.config.temp;
	}

	getCompiler(): ITestCompilerManager {
		if (!this.compiler) {
			this.compiler = new TestCompilerManager();
		}
		return this.compiler;
	}

	getRunner(file: string, env: ITestEnv): ITestRunner {
		if (!this.config.runnerCreator) {
			throw new Error("TestContext: Runner creator not found");
		}

		const runnerKey = this.config.runnerCreator.key(
			this,
			this.config.name,
			file
		);
		let runner = this.runners.get(runnerKey);
		if (runner) {
			return runner;
		}
		runner = this.config.runnerCreator.runner(
			this,
			this.config.name,
			file,
			env
		);
		this.runners.set(runnerKey, runner!);
		return runner;
	}

	getTestConfig(): TTestConfig {
		return this.config.testConfig || {};
	}

	setValue<T>(key: string, value: T) {
		this.store.set(key, value);
	}

	getValue<T>(key: string): T | void {
		return this.store.get(key) as T | void;
	}
	hasError(): boolean {
		return this.errors.length > 0;
	}
	emitError(err: Error | string): void {
		this.errors.push(typeof err === "string" ? new Error(err) : err);
	}
	getError(): Error[] {
		return this.errors;
	}
	clearError() {
		this.errors.length = 0;
	}
	async closeCompiler() {
		if (this.compiler) {
			await this.compiler.close();
		}
	}
}
