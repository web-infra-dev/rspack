import fs from "node:fs";
import path from "node:path";
import { rimrafSync } from "rimraf";

import createLazyTestEnv from "../helper/legacy/createLazyTestEnv";
import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestProcessor,
	ITester,
	TRunnerFactory,
	TTestConfig
} from "../type";
import { Tester } from "./tester";

declare global {
	var testFilter: string | undefined;
}

interface IConcurrentTestEnv {
	clear: () => void;
	run: () => Promise<void>;
}

export interface IBasicCaseCreatorOptions<T extends ECompilerType> {
	clean?: boolean;
	describe?: boolean;
	timeout?: number;
	contextValue?: Record<string, unknown>;
	steps: (
		creatorConfig: IBasicCaseCreatorOptions<T> & {
			name: string;
			src: string;
			dist: string;
			temp: string | void;
		}
	) => ITestProcessor[];
	testConfig?: (testConfig: TTestConfig<T>) => void;
	description?: (name: string, step: number) => string;
	runner?: new (
		name: string,
		context: ITestContext
	) => TRunnerFactory<ECompilerType>;
	[key: string]: unknown;
	concurrent?: boolean | number;
}

const DEFAULT_MAX_CONCURRENT = 5;

export class BasicCaseCreator<T extends ECompilerType> {
	protected currentConcurrent = 0;
	protected tasks: [string, () => void][] = [];

	constructor(protected _options: IBasicCaseCreatorOptions<T>) {}

	create(name: string, src: string, dist: string, temp?: string) {
		const testConfig = this.readTestConfig(src);
		if (typeof this._options.testConfig === "function") {
			this._options.testConfig(testConfig);
		}
		const skipped = this.checkSkipped(src, testConfig);
		if (skipped) {
			this.skip(name, skipped);
			return;
		}

		if (this._options.clean) {
			this.clean([dist, temp || ""].filter(Boolean));
		}

		const run = this.shouldRun(name);
		const tester = this.createTester(name, src, dist, temp, testConfig);
		const concurrent =
			testConfig.concurrent ?? this._options.concurrent ?? false;

		if (this._options.describe) {
			if (run) {
				if (concurrent) {
					describe(name, () =>
						this.describeConcurrent(name, tester, testConfig)
					);
				} else {
					describe(name, () => this.describe(name, tester, testConfig));
				}
			} else {
				describe.skip(name, () => {
					it.skip("skipped", () => {});
				});
			}
		} else {
			if (run) {
				if (concurrent) {
					this.describeConcurrent(name, tester, testConfig);
				} else {
					this.describe(name, tester, testConfig);
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
		testConfig: TTestConfig<T>
	) {
		beforeAll(async () => {
			await tester.prepare();
		});

		let starter = null;
		let chain = new Promise<void>((resolve, reject) => {
			starter = resolve;
		});
		const ender = this.registerConcurrentTask(name, starter!);
		const env = this.createConcurrentEnv();
		let bailout = false;
		for (let index = 0; index < tester.total; index++) {
			let stepSignalResolve = null;
			let stepSignalReject = null;
			const stepSignal = new Promise((resolve, reject) => {
				stepSignalResolve = resolve;
				stepSignalReject = reject;
			});
			const description =
				typeof this._options.description === "function"
					? this._options.description(name, index)
					: index
						? `step [${index}] should pass`
						: "should pass";
			it(
				description,
				async () => {
					await stepSignal;
				},
				this._options.timeout || 180000
			);

			chain = chain.then(async () => {
				try {
					if (bailout) {
						throw `Case "${name}" step ${index + 1} bailout because ${tester.step + 1} failed`;
					}
					env.clear();
					await tester.compile();
					await tester.check(env);
					await env.run();
					const context = tester.getContext();
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
					stepSignalResolve!();
				} catch (e) {
					stepSignalReject!(e);
				}
			});
		}

		chain.finally(() => {
			ender();
		});

		afterAll(async () => {
			await tester.resume();
		});
	}

	protected describe(
		name: string,
		tester: ITester,
		testConfig: TTestConfig<T>
	) {
		beforeAll(async () => {
			await tester.prepare();
		});

		let bailout = false;
		for (let index = 0; index < tester.total; index++) {
			const description =
				typeof this._options.description === "function"
					? this._options.description(name, index)
					: `step ${index ? `[${index}]` : ""} should pass`;
			it(
				description,
				async () => {
					if (bailout) {
						throw `Case "${name}" step ${index + 1} bailout because ${tester.step + 1} failed`;
					}
					await tester.compile();
					await tester.check(env);
					const context = tester.getContext();
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
				this._options.timeout || 30000
			);
			const env = this.createEnv(testConfig);
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
					} catch (e) {
						throw new Error(
							`Error: ${description} failed\n${(e as Error).stack}`
						);
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
			jest
		};
	}

	protected createEnv(testConfig: TTestConfig<T>): ITestEnv {
		if (typeof this._options.runner === "function" && !testConfig.noTest) {
			return createLazyTestEnv(10000);
		}
		return {
			expect,
			it,
			beforeEach,
			afterEach,
			jest
		};
	}

	protected clean(folders: string[]) {
		for (const f of folders) {
			rimrafSync(f);
			fs.mkdirSync(f, { recursive: true });
		}
	}

	protected skip(name: string, reason: string | boolean) {
		describe.skip(name, () => {
			it(
				typeof reason === "string" ? `filtered by ${reason}` : "filtered",
				() => {}
			);
		});
	}

	protected readTestConfig(src: string): TTestConfig<T> {
		const testConfigFile = path.join(src, "test.config.js");
		return fs.existsSync(testConfigFile) ? require(testConfigFile) : {};
	}

	protected checkSkipped(
		src: string,
		testConfig: TTestConfig<T>
	): boolean | string {
		const filterPath = path.join(src, "test.filter.js");
		return (
			fs.existsSync(filterPath) &&
			!require(filterPath)(this._options, testConfig)
		);
	}

	protected createTester(
		name: string,
		src: string,
		dist: string,
		temp: string | void,
		testConfig: TTestConfig<T>
	): ITester {
		return new Tester({
			name,
			src,
			dist,
			testConfig,
			contextValue: this._options.contextValue,
			runnerFactory: this._options.runner,
			steps: this._options.steps({
				...this._options,
				name,
				src,
				dist,
				temp
			})
		});
	}

	protected tryRunTask() {
		while (
			this.tasks.length !== 0 &&
			this.currentConcurrent < this.getMaxConcurrent()
		) {
			const [_name, starter] = this.tasks.shift()!;
			this.currentConcurrent++;
			starter();
		}
	}

	protected getMaxConcurrent() {
		return typeof this._options.concurrent === "number"
			? this._options.concurrent
			: DEFAULT_MAX_CONCURRENT;
	}

	protected registerConcurrentTask(name: string, starter: () => void) {
		this.tasks.push([name, starter]);
		this.tryRunTask();
		return () => {
			this.currentConcurrent--;
			this.tryRunTask();
		};
	}
}
