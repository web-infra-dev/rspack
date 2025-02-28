/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/MultiCompiler.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import * as liteTapable from "@rspack/lite-tapable";
import type { Compiler, RspackOptions, Stats } from ".";
import MultiStats from "./MultiStats";
import MultiWatching from "./MultiWatching";
import type { WatchOptions } from "./config";
import ConcurrentCompilationError from "./error/ConcurrentCompilationError";
import ArrayQueue from "./util/ArrayQueue";
import asyncLib from "./util/asyncLib";
import type {
	InputFileSystem,
	IntermediateFileSystem,
	WatchFileSystem
} from "./util/fs";

interface Node<T> {
	compiler: Compiler;
	children: Node<T>[];
	parents: Node<T>[];
	setupResult?: T;
	result?: Stats;
	state:
		| "pending"
		| "blocked"
		| "queued"
		| "starting"
		| "running"
		| "running-outdated"
		| "done";
}

export interface MultiCompilerOptions {
	/**
	 * how many Compilers are allows to run at the same time in parallel
	 */
	parallelism?: number;
}

export type MultiRspackOptions = ReadonlyArray<RspackOptions> &
	MultiCompilerOptions;

export class MultiCompiler {
	compilers: Compiler[];
	dependencies: WeakMap<Compiler, string[]>;
	hooks: {
		done: liteTapable.SyncHook<MultiStats>;
		invalid: liteTapable.MultiHook<
			liteTapable.SyncHook<[string | null, number]>
		>;
		run: liteTapable.MultiHook<liteTapable.AsyncSeriesHook<[Compiler]>>;
		watchClose: liteTapable.SyncHook<[]>;
		watchRun: liteTapable.MultiHook<liteTapable.AsyncSeriesHook<[Compiler]>>;
		infrastructureLog: liteTapable.MultiHook<
			liteTapable.SyncBailHook<[string, string, any[]], true>
		>;
	};
	_options: MultiCompilerOptions;
	running: boolean;

	constructor(
		compilers: Compiler[] | Record<string, Compiler>,
		options?: MultiCompilerOptions
	) {
		let normalizedCompilers: Compiler[];
		if (!Array.isArray(compilers)) {
			normalizedCompilers = Object.entries(compilers).map(
				([name, compiler]) => {
					compiler.name = name;
					return compiler;
				}
			);
		} else {
			normalizedCompilers = compilers;
		}

		this.hooks = {
			done: new liteTapable.SyncHook(["stats"]),
			invalid: new liteTapable.MultiHook(
				normalizedCompilers.map(c => c.hooks.invalid)
			),
			run: new liteTapable.MultiHook(normalizedCompilers.map(c => c.hooks.run)),
			watchClose: new liteTapable.SyncHook([]),
			watchRun: new liteTapable.MultiHook(
				normalizedCompilers.map(c => c.hooks.watchRun)
			),
			infrastructureLog: new liteTapable.MultiHook(
				normalizedCompilers.map(c => c.hooks.infrastructureLog)
			)
		};
		this.compilers = normalizedCompilers;
		this._options = {
			parallelism: options?.parallelism || Number.POSITIVE_INFINITY
		};
		this.dependencies = new WeakMap();
		this.running = false;

		const compilerStats: (Stats | null)[] = this.compilers.map(() => null);
		let doneCompilers = 0;
		for (let index = 0; index < this.compilers.length; index++) {
			const compiler = this.compilers[index];
			const compilerIndex = index;
			let compilerDone = false;
			compiler.hooks.done.tap("MultiCompiler", stats => {
				if (!compilerDone) {
					compilerDone = true;
					doneCompilers++;
				}
				compilerStats[compilerIndex] = stats;
				if (doneCompilers === this.compilers.length) {
					this.hooks.done.call(new MultiStats(compilerStats as Stats[]));
				}
			});
			compiler.hooks.invalid.tap("MultiCompiler", () => {
				if (compilerDone) {
					compilerDone = false;
					doneCompilers--;
				}
			});
		}
	}

	get options() {
		return Object.assign(
			this.compilers.map(c => c.options),
			this._options
		);
	}

	get outputPath() {
		let commonPath = this.compilers[0].outputPath;
		for (const compiler of this.compilers) {
			while (
				compiler.outputPath.indexOf(commonPath) !== 0 &&
				/[/\\]/.test(commonPath)
			) {
				commonPath = commonPath.replace(/[/\\][^/\\]*$/, "");
			}
		}

		if (!commonPath && this.compilers[0].outputPath[0] === "/") return "/";
		return commonPath;
	}

	get inputFileSystem() {
		throw new Error("Cannot read inputFileSystem of a MultiCompiler");
	}

	get outputFileSystem() {
		throw new Error("Cannot read outputFileSystem of a MultiCompiler");
	}

	get watchFileSystem() {
		throw new Error("Cannot read watchFileSystem of a MultiCompiler");
	}

	get intermediateFileSystem() {
		throw new Error("Cannot read outputFileSystem of a MultiCompiler");
	}

	set inputFileSystem(value: InputFileSystem) {
		for (const compiler of this.compilers) {
			compiler.inputFileSystem = value;
		}
	}

	set outputFileSystem(value: typeof import("fs")) {
		for (const compiler of this.compilers) {
			compiler.outputFileSystem = value;
		}
	}

	set watchFileSystem(value: WatchFileSystem) {
		for (const compiler of this.compilers) {
			compiler.watchFileSystem = value;
		}
	}

	set intermediateFileSystem(value: IntermediateFileSystem) {
		for (const compiler of this.compilers) {
			compiler.intermediateFileSystem = value;
		}
	}

	getInfrastructureLogger(name: string) {
		return this.compilers[0].getInfrastructureLogger(name);
	}

	/**
	 * @param compiler - the child compiler
	 * @param dependencies - its dependencies
	 */
	setDependencies(compiler: Compiler, dependencies: string[]) {
		this.dependencies.set(compiler, dependencies);
	}

	/**
	 * @param callback - signals when the validation is complete
	 * @returns true if the dependencies are valid
	 */
	validateDependencies(
		callback: liteTapable.Callback<Error, MultiStats>
	): boolean {
		type Edge = { source: Compiler; target: Compiler };
		const edges = new Set<Edge>();
		const missing: string[] = [];
		const targetFound = (compiler: Compiler) => {
			for (const edge of edges) {
				if (edge.target === compiler) {
					return true;
				}
			}
			return false;
		};
		const sortEdges = (e1: Edge, e2: Edge) => {
			return (
				e1.source.name!.localeCompare(e2.source.name!) ||
				e1.target.name!.localeCompare(e2.target.name!)
			);
		};
		for (const source of this.compilers) {
			const dependencies = this.dependencies.get(source);
			if (dependencies) {
				for (const dep of dependencies) {
					const target = this.compilers.find(c => c.name === dep);
					if (!target) {
						missing.push(dep);
					} else {
						edges.add({
							source,
							target
						});
					}
				}
			}
		}
		const errors: string[] = missing.map(
			m => `Compiler dependency \`${m}\` not found.`
		);
		const stack = this.compilers.filter(c => !targetFound(c));
		while (stack.length > 0) {
			const current = stack.pop();
			for (const edge of edges) {
				if (edge.source === current) {
					edges.delete(edge);
					const target = edge.target;
					if (!targetFound(target)) {
						stack.push(target);
					}
				}
			}
		}
		if (edges.size > 0) {
			const lines: string[] = Array.from(edges)
				.sort(sortEdges)
				.map(edge => `${edge.source.name} -> ${edge.target.name}`);
			lines.unshift("Circular dependency found in compiler dependencies.");
			errors.unshift(lines.join("\n"));
		}
		if (errors.length > 0) {
			const message = errors.join("\n");
			callback(new Error(message));
			return false;
		}
		return true;
	}

	/**
	 * @template SetupResult
	 * @param {function(Compiler, number, Callback<Stats>, function(): boolean, function(): void, function(): void): SetupResult} setup setup a single compiler
	 * @param {function(Compiler, SetupResult, Callback<Stats>): void} run run/continue a single compiler
	 * @param {Callback<MultiStats>} callback callback when all compilers are done, result includes Stats of all changed compilers
	 * @returns {SetupResult[]} result of setup
	 */
	#runGraph<SetupResult>(
		setup: (
			compiler?: Compiler,
			idx?: number,
			done?: liteTapable.Callback<Error, Stats>,
			isBlocked?: () => boolean,
			setChanged?: () => void,
			setInvalid?: () => void
		) => SetupResult,
		run: (
			compiler: Compiler,
			res: SetupResult,
			done: liteTapable.Callback<Error, Stats>
		) => void,
		callback: liteTapable.Callback<Error, MultiStats>
	): SetupResult[] {
		// State transitions for nodes:
		// -> blocked (initial)
		// blocked -> starting [running++] (when all parents done)
		// queued -> starting [running++] (when processing the queue)
		// starting -> running (when run has been called)
		// running -> done [running--] (when compilation is done)
		// done -> pending (when invalidated from file change)
		// pending -> blocked [add to queue] (when invalidated from aggregated changes)
		// done -> blocked [add to queue] (when invalidated, from parent invalidation)
		// running -> running-outdated (when invalidated, either from change or parent invalidation)
		// running-outdated -> blocked [running--] (when compilation is done)
		const nodes: Node<SetupResult>[] = this.compilers.map(compiler => ({
			compiler,
			setupResult: undefined,
			result: undefined,
			state: "blocked",
			children: [],
			parents: []
		}));
		// only useful for MultiCompiler options.name and options.dependencies
		const compilerToNode = new Map<string | undefined, Node<SetupResult>>();
		for (const node of nodes) compilerToNode.set(node.compiler.name, node);
		for (const node of nodes) {
			const dependencies = this.dependencies.get(node.compiler);
			if (!dependencies) continue;
			for (const dep of dependencies) {
				const parent = compilerToNode.get(dep)!;
				node.parents.push(parent);
				parent.children.push(node);
			}
		}

		const queue = new ArrayQueue<Node<SetupResult>>();
		for (const node of nodes) {
			if (node.parents.length === 0) {
				node.state = "queued";
				queue.enqueue(node);
			}
		}
		let errored = false;
		let running = 0;
		const parallelism = this._options.parallelism!;

		const nodeDone = (
			node: Node<SetupResult>,
			err: Error,
			stats: Stats
		): void => {
			if (errored) return;
			if (err) {
				errored = true;
				return asyncLib.each(
					nodes,
					(node, callback) => {
						if (node.compiler.watching) {
							node.compiler.watching.close(callback);
						} else {
							callback();
						}
					},
					() => callback(err)
				);
			}
			node.result = stats;
			running--;
			if (node.state === "running") {
				node.state = "done";
				for (const child of node.children) {
					if (child.state === "blocked") queue.enqueue(child);
				}
			} else if (node.state === "running-outdated") {
				node.state = "blocked";
				queue.enqueue(node);
			}
			processQueue();
		};
		/**
		 * @param {Node} node node
		 * @returns {void}
		 */
		const nodeInvalidFromParent = (node: Node<SetupResult>): void => {
			if (node.state === "done") {
				node.state = "blocked";
			} else if (node.state === "running") {
				node.state = "running-outdated";
			}
			for (const child of node.children) {
				nodeInvalidFromParent(child);
			}
		};
		/**
		 * @param {Node} node node
		 * @returns {void}
		 */
		const nodeInvalid = (node: Node<SetupResult>): void => {
			if (node.state === "done") {
				node.state = "pending";
			} else if (node.state === "running") {
				node.state = "running-outdated";
			}
			for (const child of node.children) {
				nodeInvalidFromParent(child);
			}
		};
		/**
		 * @param {Node} node node
		 * @returns {void}
		 */
		const nodeChange = (node: Node<SetupResult>): void => {
			nodeInvalid(node);
			if (node.state === "pending") {
				node.state = "blocked";
			}
			if (node.state === "blocked") {
				queue.enqueue(node);
				processQueue();
			}
		};

		const setupResults: SetupResult[] = [];
		nodes.forEach((node, i) => {
			setupResults.push(
				(node.setupResult = setup(
					node.compiler,
					i,
					nodeDone.bind(null, node) as liteTapable.Callback<Error, Stats>,
					() => node.state !== "starting" && node.state !== "running",
					() => nodeChange(node),
					() => nodeInvalid(node)
				))
			);
		});
		let processing = true;
		const processQueue = () => {
			if (processing) return;
			processing = true;
			process.nextTick(processQueueWorker);
		};
		const processQueueWorker = () => {
			while (running < parallelism && queue.length > 0 && !errored) {
				const node = queue.dequeue()!;
				if (
					node.state === "queued" ||
					(node.state === "blocked" &&
						node.parents.every(p => p.state === "done"))
				) {
					running++;
					node.state = "starting";
					run(
						node.compiler,
						node.setupResult!,
						nodeDone.bind(null, node) as liteTapable.Callback<Error, Stats>
					);
					node.state = "running";
				}
			}
			processing = false;
			if (
				!errored &&
				running === 0 &&
				nodes.every(node => node.state === "done")
			) {
				const stats: Stats[] = [];
				for (const node of nodes) {
					const result = node.result;
					if (result) {
						node.result = undefined;
						stats.push(result);
					}
				}
				if (stats.length > 0) {
					callback(null, new MultiStats(stats));
				}
			}
		};
		processQueueWorker();
		return setupResults;
	}

	/**
	 * @param watchOptions - the watcher's options
	 * @param handler - signals when the call finishes
	 * @returns a compiler watcher
	 */
	watch(
		watchOptions: WatchOptions,
		handler: liteTapable.Callback<Error, MultiStats>
	): MultiWatching {
		if (this.running) {
			return handler(new ConcurrentCompilationError()) as never;
		}
		this.running = true;

		if (this.validateDependencies(handler)) {
			const watchings = this.#runGraph(
				(compiler, idx, done, isBlocked, setChanged, setInvalid) => {
					const watching = compiler!.watch(
						Array.isArray(watchOptions) ? watchOptions[idx!] : watchOptions,
						done!
					);
					if (watching) {
						watching.onInvalid = setInvalid!;
						watching.onChange = setChanged!;
						watching.isBlocked = isBlocked!;
					}
					return watching;
				},
				(compiler, watching, _done) => {
					if (compiler.watching !== watching) return;
					if (!watching.running) watching.invalidate();
				},
				handler
			);
			return new MultiWatching(watchings, this);
		}

		return new MultiWatching([], this);
	}

	/**
	 * @param callback - signals when the call finishes
	 * @param options - additional data like modifiedFiles, removedFiles
	 */
	run(
		callback: liteTapable.Callback<Error, MultiStats>,
		options?: {
			modifiedFiles?: ReadonlySet<string>;
			removedFiles?: ReadonlySet<string>;
		}
	) {
		if (this.running) {
			return callback(new ConcurrentCompilationError());
		}
		this.running = true;

		if (this.validateDependencies(callback)) {
			this.#runGraph(
				() => {},
				(compiler, _, callback) => compiler.run(callback, options),
				(err, stats) => {
					this.running = false;

					if (callback !== undefined) {
						return callback(err, stats);
					}
				}
			);
		}
	}

	purgeInputFileSystem() {
		for (const compiler of this.compilers) {
			compiler.inputFileSystem?.purge?.();
		}
	}

	close(callback: liteTapable.Callback<Error, void>) {
		asyncLib.each(
			this.compilers,
			(compiler, cb) => {
				compiler.close(cb);
			},
			// cannot be resolved without assertion
			// Type 'Error | null | undefined' is not assignable to type 'Error | null'
			callback as (err: Error | null | undefined) => void
		);
	}
}
