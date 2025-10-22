import EventEmitter from "node:events";
import type { Compiler, RspackOptions, Stats } from "@rspack/core";
import merge from "webpack-merge";
import type { ITestCompilerManager } from "./type";

export enum ECompilerEvent {
	Build = "build",
	Option = "option",
	Create = "create",
	Close = "close"
}

export class TestCompilerManager implements ITestCompilerManager {
	protected compilerOptions: RspackOptions = {} as RspackOptions;
	protected compilerInstance: Compiler | null = null;
	protected compilerStats: Stats | null = null;
	protected emitter: EventEmitter = new EventEmitter();

	constructor() {}

	getOptions(): RspackOptions {
		return this.compilerOptions;
	}

	setOptions(newOptions: RspackOptions): RspackOptions {
		this.compilerOptions = newOptions;
		this.emitter.emit(ECompilerEvent.Option, this.compilerOptions);
		return this.compilerOptions;
	}

	mergeOptions(newOptions: RspackOptions): RspackOptions {
		this.compilerOptions = merge(this.compilerOptions, newOptions);
		this.emitter.emit(ECompilerEvent.Option, this.compilerOptions);
		return this.compilerOptions;
	}

	getCompiler(): Compiler | null {
		return this.compilerInstance;
	}

	createCompiler(): Compiler {
		this.compilerInstance = require("@rspack/core")(
			this.compilerOptions
		) as Compiler;
		this.emitter.emit(ECompilerEvent.Create, this.compilerInstance);
		return this.compilerInstance;
	}

	createCompilerWithCallback(
		callback: (error: Error | null, stats: Stats | null) => void
	): Compiler {
		this.compilerInstance = require("@rspack/core")(
			this.compilerOptions,
			callback
		) as Compiler;
		this.emitter.emit(ECompilerEvent.Create, this.compilerInstance);
		return this.compilerInstance;
	}

	build(): Promise<Stats> {
		if (!this.compilerInstance)
			throw new Error("Compiler should be created before build");
		return new Promise<Stats>((resolve, reject) => {
			try {
				this.compilerInstance!.run((error, newStats) => {
					this.emitter.emit(ECompilerEvent.Build, error, newStats);
					if (error) return reject(error);
					this.compilerStats = newStats as Stats;
					resolve(newStats as Stats);
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
					this.compilerStats = newStats as Stats;
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
