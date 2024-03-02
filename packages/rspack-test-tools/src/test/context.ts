import { TestCompilerManager } from "../compiler";
import {
	ITestCompilerManager,
	ECompilerType,
	ITestContext,
	ITesterConfig,
	TCompiler,
	TCompilerOptions,
	TCompilerStats,
	TTestRunResult,
	TCompilerFactory
} from "../type";
import path from "path";

export class TestContext implements ITestContext {
	protected errors: Map<string, Error[]> = new Map();
	protected compilers: Map<string, ITestCompilerManager<ECompilerType>> =
		new Map();
	protected result: Map<string, unknown> = new Map();

	constructor(private config: ITesterConfig) {}

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
		factory: TCompilerFactory<T>
	): ITestCompilerManager<T> {
		let compiler = this.compilers.get(name);
		if (!compiler) {
			if (!factory) {
				throw new Error("Compiler does not exists");
			}
			compiler = new TestCompilerManager(factory);
			this.compilers.set(name, compiler);
		}
		return compiler;
	}

	setResult<T>(name: string, value: T) {
		this.result.set(name, value);
	}

	getResult<T>(name: string): T | void {
		return this.result.get(name) as T;
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
