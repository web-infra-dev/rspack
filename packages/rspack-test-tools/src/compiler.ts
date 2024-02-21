import {
	ECompilerType,
	ITestCompilerManager,
	ITestContext,
	TCompiler,
	TCompilerOptions,
	TCompilerStats
} from "./type";
import merge from "webpack-merge";

export class TestCompilerManager<T extends ECompilerType>
	implements ITestCompilerManager<T>
{
	private compilerOptions: TCompilerOptions<T> = {} as TCompilerOptions<T>;
	private compilerInstance: TCompiler<T> | null = null;
	private compilerStats: TCompilerStats<T> | null = null;
	private runResult: unknown;

	options(
		context: ITestContext,
		fn: (options: TCompilerOptions<T>) => TCompilerOptions<T> | void
	) {
		try {
			const newOptions = fn(this.compilerOptions);
			if (newOptions) {
				this.compilerOptions = merge(this.compilerOptions, newOptions);
			}
		} catch (e) {
			context.emitError(e as Error);
		}
	}
	compiler(
		context: ITestContext,
		fn: (
			options: TCompilerOptions<T>,
			compiler: TCompiler<T> | null
		) => TCompiler<T> | null
	) {
		try {
			const newCompiler = fn(this.compilerOptions, this.compilerInstance);
			if (newCompiler) {
				this.compilerInstance = newCompiler;
			}
		} catch (e) {
			context.emitError(e as Error);
		}
	}
	stats(
		context: ITestContext,
		fn: (
			compiler: TCompiler<T> | null,
			stats: TCompilerStats<T> | null
		) => TCompilerStats<T> | void
	) {
		try {
			const newStats = fn(this.compilerInstance, this.compilerStats);
			if (newStats) {
				this.compilerStats = newStats;
			}
		} catch (e) {
			context.emitError(e as Error);
		}
	}
	result(
		context: ITestContext,
		fn: <R>(compiler: TCompiler<T> | null, result: R) => R
	) {
		try {
			const newResult = fn(this.compilerInstance, this.runResult);
			if (newResult) {
				this.runResult = newResult;
			}
		} catch (e) {
			context.emitError(e as Error);
		}
	}
	async build(
		context: ITestContext,
		fn: (compiler: TCompiler<T>) => Promise<void>
	) {
		if (!this.compilerInstance) {
			context.emitError(new Error("Build failed: compiler not exists"));
		}
		try {
			await fn(this.compilerInstance!);
		} catch (e) {
			context.emitError(e as Error);
		}
	}
}
