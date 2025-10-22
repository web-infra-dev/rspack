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

export type TTestContextOptions = Omit<ITesterConfig, "name" | "steps">;

export class TestContext implements ITestContext {
	protected errors: Map<string, Error[]> = new Map();
	protected compilers: Map<string, ITestCompilerManager> = new Map();
	protected store: Map<string, Record<string, unknown>> = new Map();
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

	getCompiler(name: string): ITestCompilerManager {
		let compiler = this.compilers.get(name);
		if (!compiler) {
			compiler = new TestCompilerManager();
			this.compilers.set(name, compiler);
		}
		return compiler;
	}

	getRunner(name: string, file: string, env: ITestEnv): ITestRunner {
		if (!this.config.runnerCreator) {
			throw new Error("TestContext: Runner creator not found");
		}

		const runnerKey = this.config.runnerCreator.key(this, name, file);
		let runner = this.runners.get(runnerKey);
		if (runner) {
			return runner;
		}
		runner = this.config.runnerCreator.runner(this, name, file, env);
		this.runners.set(runnerKey, runner!);
		return runner;
	}

	getTestConfig(): TTestConfig {
		return this.config.testConfig || {};
	}

	setValue<T>(name: string, key: string, value: T) {
		if (!this.store.has(name)) {
			this.store.set(name, {});
		}
		const scope = this.store.get(name)!;
		scope[key] = value;
	}

	getValue<T>(name: string, key: string): T | void {
		if (!this.store.has(name)) {
			this.store.set(name, {});
		}
		const scope = this.store.get(name)!;
		return scope[key] as T | void;
	}

	hasError(name?: string): boolean {
		if (name) {
			return this.getError(name).length > 0;
		}
		return !!Array.from(this.errors.values()).reduce(
			(res, arr) => res + arr.length,
			0
		);
	}
	emitError(name: string, err: Error | string): void {
		const errors = this.errors.get(name) || [];
		errors.push(typeof err === "string" ? new Error(err) : err);
		this.errors.set(name, errors);
	}
	getNames() {
		return Array.from(this.compilers.keys());
	}
	getError(name?: string): Error[] {
		if (name) {
			return this.errors.get(name) || [];
		}
		return Array.prototype.concat(...this.errors.values());
	}
	clearError(name?: string) {
		if (name) {
			this.errors.delete(name);
		} else {
			this.errors.clear();
		}
	}
	async closeCompiler(name: string) {
		const compiler = this.getCompiler(name);
		if (compiler) {
			await compiler.close();
		}
	}
}
