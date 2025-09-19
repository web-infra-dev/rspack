import EventEmitter from "node:events";
import merge from "webpack-merge";

import {
	ECompilerType,
	type ITestCompilerManager,
	type TCompiler,
	type TCompilerFactories,
	type TCompilerFactory,
	type TCompilerOptions,
	type TCompilerStats
} from "./type";

export enum ECompilerEvent {
	Build = "build",
	Option = "option",
	Create = "create",
	Close = "close"
}

export const COMPILER_FACTORIES: TCompilerFactories<ECompilerType> = {
	[ECompilerType.Rspack]: ((
		options: TCompilerOptions<ECompilerType.Rspack>,
		callback?: (
			error: Error | null,
			stats: TCompilerStats<ECompilerType.Rspack> | null
		) => void
	) =>
		require("@rspack/core")(
			options,
			callback
		)) as TCompilerFactory<ECompilerType>,
	[ECompilerType.Webpack]: ((
		options: TCompilerOptions<ECompilerType.Webpack>,
		callback?: (
			error: Error | null,
			stats: TCompilerStats<ECompilerType.Webpack> | null
		) => void
	) => require("webpack")(options, callback)) as TCompilerFactory<ECompilerType>
};
export class TestCompilerManager<T extends ECompilerType>
	implements ITestCompilerManager<T>
{
	protected compilerOptions: TCompilerOptions<T> = {} as TCompilerOptions<T>;
	protected compilerInstance: TCompiler<T> | null = null;
	protected compilerStats: TCompilerStats<T> | null = null;
	protected emitter: EventEmitter = new EventEmitter();

	constructor(
		protected type: T,
		protected factories: TCompilerFactories<T> = COMPILER_FACTORIES
	) {}

	getOptions(): TCompilerOptions<T> {
		return this.compilerOptions;
	}

	setOptions(newOptions: TCompilerOptions<T>): TCompilerOptions<T> {
		this.compilerOptions = newOptions;
		this.emitter.emit(ECompilerEvent.Option, this.compilerOptions);
		return this.compilerOptions;
	}

	mergeOptions(newOptions: TCompilerOptions<T>): TCompilerOptions<T> {
		this.compilerOptions = merge(this.compilerOptions, newOptions);
		this.emitter.emit(ECompilerEvent.Option, this.compilerOptions);
		return this.compilerOptions;
	}

	getCompiler(): TCompiler<T> | null {
		return this.compilerInstance;
	}

	createCompiler(): TCompiler<T> {
		this.compilerInstance = this.factories[this.type](
			this.compilerOptions
		) as TCompiler<T>;
		this.emitter.emit(ECompilerEvent.Create, this.compilerInstance);
		return this.compilerInstance;
	}

	createCompilerWithCallback(
		callback: (error: Error | null, stats: TCompilerStats<T> | null) => void
	): TCompiler<T> {
		this.compilerInstance = this.factories[this.type](
			this.compilerOptions,
			callback
		) as TCompiler<T>;
		this.emitter.emit(ECompilerEvent.Create, this.compilerInstance);
		return this.compilerInstance;
	}

	build(): Promise<TCompilerStats<T>> {
		if (!this.compilerInstance)
			throw new Error("Compiler should be created before build");
		return new Promise<TCompilerStats<T>>((resolve, reject) => {
			try {
				this.compilerInstance!.run((error, newStats) => {
					this.emitter.emit(ECompilerEvent.Build, error, newStats);
					if (error) return reject(error);
					this.compilerStats = newStats as TCompilerStats<T>;
					resolve(newStats as TCompilerStats<T>);
				});
			} catch (e) {
				reject(e);
			}
		});
	}

	watch(timeout = 1000) {
		if (!this.compilerInstance)
			throw new Error("Compiler should be created before watch");
		this.compilerInstance!.watch(
			{
				// IMPORTANT:
				// This is a workaround for the issue that watchpack cannot detect the file change in time
				// so we set the poll to 300ms to make it more sensitive to the file change
				poll: 300,
				// Rspack ignored node_modules and .git by default for better performance, but for tests we
				// want to watch all files, which aligns with webpack's default behavior
				ignored: [],
				aggregateTimeout: timeout
			},
			(error, newStats) => {
				this.emitter.emit(ECompilerEvent.Build, error, newStats);
				if (error) return error;
				if (newStats) {
					this.compilerStats = newStats as TCompilerStats<T>;
				}
				return newStats;
			}
		);
	}

	getStats() {
		return this.compilerStats;
	}

	getEmitter() {
		return this.emitter;
	}

	close(): Promise<void> {
		return new Promise<void>((resolve, reject) => {
			if (this.compilerInstance) {
				this.compilerInstance.close(e => {
					this.emitter.emit(ECompilerEvent.Close, e);
					e ? reject(e) : resolve();
				});
			} else {
				resolve();
			}
		});
	}
}
