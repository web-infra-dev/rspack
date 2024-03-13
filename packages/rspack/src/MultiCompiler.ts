/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/MultiCompiler.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import { Compiler, RspackOptions, Stats } from ".";
import ResolverFactory from "./ResolverFactory";
import { WatchFileSystem } from "./util/fs";
import { Watching } from "./Watching";
import { AsyncSeriesHook, Callback, MultiHook, SyncHook } from "tapable";
import MultiStats from "./MultiStats";
import asyncLib from "neo-async";
import ArrayQueue from "./util/ArrayQueue";
import ConcurrentCompilationError from "./error/ConcurrentCompilationError";
import MultiWatching from "./MultiWatching";
import { WatchOptions } from "./config";

type Any = any;

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
	// @ts-expect-error
	context: string;
	compilers: Compiler[];
	dependencies: WeakMap<Compiler, string[]>;
	hooks: {
		done: SyncHook<MultiStats>;
		invalid: MultiHook<SyncHook<[string | null, number]>>;
		run: MultiHook<AsyncSeriesHook<[Compiler]>>;
		watchClose: SyncHook<Any>;
		watchRun: MultiHook<Any>;
		infrastructureLog: MultiHook<Any>;
	};
	// @ts-expect-error
	name: string;
	infrastructureLogger: Any;
	_options: { parallelism?: number };
	// @ts-expect-error
	root: Compiler;
	// @ts-expect-error
	resolverFactory: ResolverFactory;
	running: boolean;
	// @ts-expect-error
	watching: Watching;
	// @ts-expect-error
	watchMode: boolean;

	constructor(
		compilers: Compiler[] | Record<string, Compiler>,
		options?: MultiCompilerOptions
	) {
		if (!Array.isArray(compilers)) {
			compilers = Object.entries(compilers).map(([name, compiler]) => {
				compiler.name = name;
				return compiler;
			});
		}

		this.hooks = {
			/** @type {SyncHook<[MultiStats]>} */
			done: new SyncHook(["stats"]),
			/** @type {MultiHook<SyncHook<[string | null, number]>>} */
			invalid: new MultiHook(compilers.map(c => c.hooks.invalid)),
			/** @type {MultiHook<AsyncSeriesHook<[Compiler]>>} */
			run: new MultiHook(compilers.map(c => c.hooks.run)),
			/** @type {SyncHook<[]>} */
			watchClose: new SyncHook([]),
			/** @type {MultiHook<AsyncSeriesHook<[Compiler]>>} */
			watchRun: new MultiHook(compilers.map(c => c.hooks.watchRun)),
			/** @type {MultiHook<SyncBailHook<[string, string, any[]], true>>} */
			infrastructureLog: new MultiHook(
				compilers.map(c => c.hooks.infrastructureLog)
			)
		};
		this.compilers = compilers;
		this._options = {
			parallelism: options?.parallelism || Infinity
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

	/**
	 * @param {InputFileSystem} value the new input file system
	 */
	set inputFileSystem(value) {
		for (const compiler of this.compilers) {
			compiler.inputFileSystem = value;
		}
	}

	/**
	 * @param {OutputFileSystem} value the new output file system
	 */
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

	/**
	 * @param {IntermediateFileSystem} value the new intermediate file system
	 */
	set intermediateFileSystem(value: any) {
		for (const compiler of this.compilers) {
			compiler.intermediateFileSystem = value;
		}
	}

	getInfrastructureLogger(name: string) {
		return this.compilers[0].getInfrastructureLogger(name);
	}

	/**
	 * @param {Compiler} compiler the child compiler
	 * @param {string[]} dependencies its dependencies
	 * @returns {void}
	 */
	setDependencies(compiler: Compiler, dependencies: string[]) {
		this.dependencies.set(compiler, dependencies);
	}

	/**
	 * @param {Callback<MultiStats>} callback signals when the validation is complete
	 * @returns {boolean} true if the dependencies are valid
	 */
	validateDependencies(callback: Callback<Error, MultiStats>): boolean {
		const edges = new Set<{ source: Compiler; target: Compiler }>();
		const missing: string[] = [];
		const targetFound = (compiler: Compiler) => {
			for (const edge of edges) {
				if (edge.target === compiler) {
					return true;
				}
			}
			return false;
		};
		// @ts-expect-error
		const sortEdges = (e1, e2) => {
			return (
				e1.source.name.localeCompare(e2.source.name) ||
				e1.target.name.localeCompare(e2.target.name)
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
		/** @type {string[]} */
		const errors = missing.map(m => `Compiler dependency \`${m}\` not found.`);
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
			/** @type {string[]} */
			const lines = Array.from(edges)
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
			done?: Callback<Error, Stats>,
			isBlocked?: () => boolean,
			setChanged?: () => void,
			setInvalid?: () => void
		) => SetupResult,
		run: (
			compiler: Compiler,
			res: SetupResult,
			done: Callback<Error, Stats>
		) => void,
		callback: Callback<Error, MultiStats>
	): SetupResult[] {
		/** @typedef {{ compiler: Compiler, setupResult: SetupResult, result: Stats, state: "pending" | "blocked" | "queued" | "starting" | "running" | "running-outdated" | "done", children: Node[], parents: Node[] }} Node */

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
		/**
		 * @param {Node} node node
		 * @param {Error=} err error
		 * @param {Stats=} stats result
		 * @returns {void}
		 */
		const nodeDone = (node: Node<SetupResult>, err: Error, stats: Stats) => {
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
		const nodeInvalidFromParent = (node: Node<SetupResult>) => {
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
		const nodeInvalid = (node: Node<SetupResult>) => {
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
		// @ts-expect-error
		const nodeChange = node => {
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
					// @ts-expect-error
					nodeDone.bind(null, node),
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
					// @ts-expect-error
					run(node.compiler, node.setupResult!, nodeDone.bind(null, node));
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
	 * @param {WatchOptions|WatchOptions[]} watchOptions the watcher's options
	 * @param {Callback<MultiStats>} handler signals when the call finishes
	 * @returns {MultiWatching} a compiler watcher
	 */
	watch(
		watchOptions: WatchOptions,
		handler: Callback<Error, MultiStats>
	): MultiWatching {
		if (this.running) {
			return handler(new ConcurrentCompilationError()) as never;
		}
		this.running = true;

		if (this.validateDependencies(handler)) {
			const watchings = this.#runGraph(
				// @ts-expect-error
				(compiler: Compiler, idx, done, isBlocked, setChanged, setInvalid) => {
					const watching = compiler.watch(
						// @ts-expect-error
						Array.isArray(watchOptions) ? watchOptions[idx] : watchOptions,
						// @ts-expect-error
						done
					);
					if (watching) {
						watching.onInvalid = setInvalid;
						watching.onChange = setChanged;
						watching.isBlocked = isBlocked;
					}
					return watching;
				},
				(compiler, watching, _done) => {
					if (compiler.watching !== watching) return;
					if (!watching?.running) watching?.invalidate();
				},
				handler
			);
			// @ts-expect-error
			return new MultiWatching(watchings, this);
		}

		return new MultiWatching([], this);
	}

	run(callback: Callback<Error, MultiStats>) {
		if (this.running) {
			return callback(new ConcurrentCompilationError());
		}
		this.running = true;

		if (this.validateDependencies(callback)) {
			this.#runGraph(
				() => {},
				(compiler, _, callback) => compiler.run(callback),
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
			if (compiler.inputFileSystem && compiler.inputFileSystem.purge) {
				compiler.inputFileSystem.purge();
			}
		}
	}

	close(callback: Callback<Error, void>) {
		asyncLib.each(
			this.compilers,
			(compiler, cb) => {
				compiler.close(cb);
			},
			// @ts-expect-error
			callback
		);
	}
}
