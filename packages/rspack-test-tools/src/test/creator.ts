import fs from "node:fs";
import path from "node:path";
import { rimrafSync } from "rimraf";

import createLazyTestEnv from "../helper/legacy/createLazyTestEnv";
import type {
	ITestContext,
	ITestEnv,
	ITester,
	ITesterConfig,
	ITestProcessor,
	TTestConfig,
	TTestRunnerCreator
} from "../type";
import { Tester } from "./tester";

declare global {
	var testFilter: string | undefined;
}

interface IConcurrentTestEnv {
	clear: () => void;
	run: () => Promise<void>;
}

export interface IBasicCaseCreatorOptions {
	clean?: boolean;
	describe?: boolean;
	timeout?: number;
	contextValue?: Record<string, unknown>;
	steps: (
		creatorConfig: IBasicCaseCreatorOptions & {
			name: string;
			src: string;
			dist: string;
			temp: string | void;
		}
	) => ITestProcessor[];
	testConfig?: (testConfig: TTestConfig) => void;
	description?: (name: string, step: number) => string;
	runner?: TTestRunnerCreator;
	createContext?: (config: ITesterConfig) => ITestContext;
	concurrent?: boolean | number;
	[key: string]: unknown;
}

const DEFAULT_MAX_CONCURRENT = process.env.WASM ? 1 : 5;

export class BasicCaseCreator {
	protected currentConcurrent = 0;
	protected tasks: [string, () => void][] = [];

	constructor(protected _options: IBasicCaseCreatorOptions) {}

	create(
		name: string,
		src: string,
		dist: string,
		temp?: string,
		caseOptions?: Partial<IBasicCaseCreatorOptions>
	) {
		const options = {
			...this._options,
			...caseOptions
		};
		const testConfig = this.readTestConfig(src);
		if (typeof options.testConfig === "function") {
			options.testConfig(testConfig);
		}

		const skipped = this.checkSkipped(src, testConfig, options);
		if (skipped) {
			this.skip(name, skipped);
			return;
		}

		if (options.clean) {
			this.clean([dist, temp || ""].filter(Boolean));
		}

		const run = this.shouldRun(name);
		const tester = this.createTester(
			name,
			src,
			dist,
			temp,
			testConfig,
			options
		);
		const concurrent = process.env.WASM
			? 1
			: testConfig.concurrent || options.concurrent;
		if (options.describe) {
			if (run) {
				if (concurrent) {
					describe(name, () =>
						this.describeConcurrent(name, tester, testConfig, options)
					);
				} else {
					describe(name, () =>
						this.describe(name, tester, testConfig, options)
					);
				}
			} else {
				describe.skip(name, () => {
					it.skip("skipped", () => {});
				});
			}
		} else {
			if (run) {
				if (concurrent) {
					this.describeConcurrent(name, tester, testConfig, options);
				} else {
					this.describe(name, tester, testConfig, options);
				}
			} else {
				it.skip("skipped", () => {});
			}
		}

		return tester;
	}

	protected shouldRun(name: string) {
		// TODO: more flexible filter
		if (typeof global.testFilter !== "string" || !global.testFilter) {
			return true;
		}
		return name.includes(global.testFilter);
	}

	protected describeConcurrent(
		name: string,
		tester: ITester,
		testConfig: TTestConfig,
		options: IBasicCaseCreatorOptions
	) {
		beforeAll(async () => {
			await tester.prepare();
		});

		let starter = null;
		let chain = new Promise<void>((resolve, reject) => {
			starter = resolve;
		});
		const ender = this.registerConcurrentTask(
			name,
			starter!,
			options.concurrent as number
		);
		const env = this.createConcurrentEnv();
		for (let index = 0; index < tester.total; index++) {
			let stepSignalResolve = null;
			const stepSignal = new Promise<Error>((resolve, reject) => {
				stepSignalResolve = resolve;
			}).catch(() => {
				// prevent unhandled rejection
			});
			const description =
				typeof options.description === "function"
					? options.description(name, index)
					: index
						? `step [${index}] should pass`
						: "should pass";
			it(
				description,
				async () => {
					const e = await stepSignal;
					if (e) {
						throw e;
					}
				},
				options.timeout || 300000
			);

			chain = chain.then(
				async () => {
					try {
						env.clear();
						await tester.compile();
						await tester.check(env);
						await env.run();
						await tester.after();
						const context = tester.getContext();
						if (!tester.next() && context.hasError()) {
							const errors = context
								.getError()
								.map(i => `${i.stack}`.split("\n").join("\t\n"))
								.join("\n\n");
							throw new Error(
								`Case "${name}" failed at step ${tester.step + 1}:\n${errors}`
							);
						}
						stepSignalResolve!();
					} catch (e) {
						stepSignalResolve!(e);
						return Promise.reject();
					}
				},
				() => {
					// bailout
					stepSignalResolve!();
					return Promise.reject();
				}
			);
		}

		chain
			.catch(() => {
				// bailout error
				// prevent unhandled rejection
			})
			.finally(() => {
				ender();
			});

		afterAll(async () => {
			await tester.resume();
		});
	}

	protected describe(
		name: string,
		tester: ITester,
		testConfig: TTestConfig,
		options: IBasicCaseCreatorOptions
	) {
		beforeAll(async () => {
			await tester.prepare();
		});

		let bailout = false;
		for (let index = 0; index < tester.total; index++) {
			const description =
				typeof options.description === "function"
					? options.description(name, index)
					: `step [${index}] should pass`;
			it(
				description,
				async () => {
					if (bailout) {
						throw `Case "${name}" step ${index + 1} bailout because ${tester.step + 1} failed`;
					}
					const context = tester.getContext();
					try {
						await tester.compile();
					} catch (e) {
						bailout = true;
						context.emitError(e as Error);
					}
					await tester.check(env);
					if (!tester.next() && context.hasError()) {
						bailout = true;
						const errors = context
							.getError()
							.map(i => `${i.stack}`.split("\n").join("\t\n"))
							.join("\n\n");

						throw new Error(
							`Case "${name}" failed at step ${tester.step + 1}:\n${errors}`
						);
					}
				},
				options.timeout || 60000
			);
			const env = this.createEnv(testConfig, options);
		}

		afterAll(async () => {
			await tester.resume();
		});
	}

	protected createConcurrentEnv(): ITestEnv & IConcurrentTestEnv {
		const tasks: [string, () => Promise<void> | void][] = [];
		const beforeTasks: (() => Promise<void> | void)[] = [];
		const afterTasks: (() => Promise<void> | void)[] = [];
		return {
			clear: () => {
				tasks.length = 0;
				beforeTasks.length = 0;
				afterTasks.length = 0;
			},
			run: async () => {
				const runFn = async (
					fn: (done?: (e?: Error) => void) => Promise<void> | void
				) => {
					if (fn.length) {
						await new Promise<void>((resolve, reject) => {
							fn(e => {
								if (e) {
									reject(e);
								} else {
									resolve();
								}
							});
						});
					} else {
						const res = fn();
						if (typeof res?.then === "function") {
							await res;
						}
					}
				};

				for (const [description, fn] of tasks) {
					for (const before of beforeTasks) {
						await runFn(before);
					}
					try {
						await runFn(fn);
					} catch (err) {
						const e = err as Error;
						const message = `Error: ${description} failed:\n${e.message}`;
						e.message = message;
						throw e;
					}
					for (const after of afterTasks) {
						await runFn(after);
					}
				}
			},
			expect,
			it: (description: string, fn: () => Promise<void> | void) => {
				expect(typeof description === "string");
				expect(typeof fn === "function");
				tasks.push([description, fn]);
			},
			beforeEach: (fn: () => Promise<void> | void) => {
				expect(typeof fn === "function");
				beforeTasks.push(fn);
			},
			afterEach: (fn: () => Promise<void> | void) => {
				expect(typeof fn === "function");
				afterTasks.push(fn);
			},
			jest: global.jest || global.rstest,
			rstest: global.rstest
		};
	}

	protected createEnv(
		testConfig: TTestConfig,
		options: IBasicCaseCreatorOptions
	): ITestEnv {
		if (options.runner && !testConfig.noTests) {
			return createLazyTestEnv(10000);
		}
		return {
			expect,
			it,
			beforeEach,
			afterEach,
			jest: global.jest || global.rstest,
			rstest: global.rstest
		};
	}

	protected clean(folders: string[]) {
		for (const f of folders) {
			rimrafSync(f);
		}
	}

	protected skip(name: string, reason: string | boolean) {
		it(
			typeof reason === "string" ? `filtered by ${reason}` : "filtered",
			() => {}
		);
	}

	protected readTestConfig(src: string): TTestConfig {
		const testConfigFile = path.join(src, "test.config.js");
		return fs.existsSync(testConfigFile) ? require(testConfigFile) : {};
	}

	protected checkSkipped(
		src: string,
		testConfig: TTestConfig,
		options: IBasicCaseCreatorOptions
	): boolean | string {
		const filterPath = path.join(src, "test.filter.js");
		// no test.filter.js, should not skip
		if (!fs.existsSync(filterPath)) {
			return false;
		}
		// test.filter.js exists, skip if it returns false|string|array
		const filtered = require(filterPath)(options, testConfig);
		if (typeof filtered === "string" || Array.isArray(filtered)) {
			return true;
		}
		return !filtered;
	}

	protected createTester(
		name: string,
		src: string,
		dist: string,
		temp: string | undefined,
		testConfig: TTestConfig,
		options: IBasicCaseCreatorOptions
	): ITester {
		return new Tester({
			name,
			src,
			dist,
			temp,
			testConfig,
			contextValue: options.contextValue,
			runnerCreator: options.runner,
			createContext: options.createContext,
			steps: options.steps({
				...options,
				name,
				src,
				dist,
				temp
			})
		});
	}

	protected tryRunTask(concurrent?: number) {
		while (
			this.tasks.length !== 0 &&
			this.currentConcurrent < this.getMaxConcurrent(concurrent)
		) {
			const [_name, starter] = this.tasks.shift()!;
			this.currentConcurrent++;
			starter();
		}
	}

	protected getMaxConcurrent(concurrent?: number) {
		return typeof concurrent === "number" ? concurrent : DEFAULT_MAX_CONCURRENT;
	}

	protected registerConcurrentTask(
		name: string,
		starter: () => void,
		concurrent?: number
	) {
		this.tasks.push([name, starter]);
		this.tryRunTask(concurrent);
		return () => {
			this.currentConcurrent--;
			this.tryRunTask(concurrent);
		};
	}
}
