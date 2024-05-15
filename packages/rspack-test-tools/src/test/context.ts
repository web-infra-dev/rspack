import path from "path";

import { TestCompilerManager } from "../compiler";
import {
	ECompilerType,
	ITestCompilerManager,
	ITestContext,
	ITesterConfig,
	ITestRunner,
	TRunnerFactory,
	TTestConfig
} from "../type";

export type TTestContextOptions = Omit<ITesterConfig, "name" | "steps">;

export class TestContext implements ITestContext {
	protected errors: Map<string, Error[]> = new Map();
	protected compilers: Map<string, ITestCompilerManager<ECompilerType>> =
		new Map();
	protected store: Map<string, Record<string, unknown>> = new Map();
	protected runners: Map<string, ITestRunner> = new Map();
	protected runnerFactory: TRunnerFactory<ECompilerType> | null = null;

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

	getCompiler<T extends ECompilerType>(
		name: string,
		type: T | void
	): ITestCompilerManager<T> {
		let compiler = this.compilers.get(name);
		if (!compiler) {
			if (!type) {
				throw new Error("Compiler does not exists");
			}
			compiler = new TestCompilerManager(type);
			this.compilers.set(name, compiler);
		}
		return compiler;
	}

	getRunnerFactory<T extends ECompilerType>(
		name: string
	): TRunnerFactory<T> | null {
		if (
			!this.runnerFactory &&
			typeof this.config.runnerFactory === "function"
		) {
			this.runnerFactory = new this.config.runnerFactory(name, this);
		}
		return this.runnerFactory;
	}

	getRunner(key: string): ITestRunner | null {
		return this.runners.get(key) || null;
	}

	setRunner(key: string, runner: ITestRunner) {
		this.runners.set(key, runner);
	}

	getTestConfig<T extends ECompilerType>(): TTestConfig<T> {
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
		return Array.from(this.errors.values()).reduce(
			(res, arr) => [...res, ...arr],
			[]
		);
	}
	clearError(name?: string) {
		if (name) {
			this.errors.delete(name);
		} else {
			this.errors.clear();
		}
	}
}
