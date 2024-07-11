import fs from "fs";
import path from "path";
import rimraf from "rimraf";

import createLazyTestEnv from "../helper/legacy/createLazyTestEnv";
import {
	type ECompilerType,
	EDocumentType,
	type ITestContext,
	type ITestEnv,
	type ITestProcessor,
	type ITester,
	type TRunnerFactory,
	type TTestConfig
} from "../type";
import { Tester } from "./tester";

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
	description?: (name: string, step: number) => string;
	runner?: new (
		name: string,
		context: ITestContext
	) => TRunnerFactory<ECompilerType>;
	[key: string]: unknown;
}

export class BasicCaseCreator<T extends ECompilerType> {
	constructor(protected _options: IBasicCaseCreatorOptions<T>) {}

	create(name: string, src: string, dist: string, temp?: string) {
		const testConfig = this.readTestConfig(src);
		const skipped = this.checkSkipped(src, testConfig);
		if (skipped) {
			this.skip(name, skipped);
			return;
		}

		if (this._options.clean) {
			this.clean([dist, temp || ""].filter(Boolean));
		}

		const tester = this.createTester(name, src, dist, temp, testConfig);

		if (this._options.describe) {
			describe(name, () => this.describe(name, tester, testConfig));
		} else {
			this.describe(name, tester, testConfig);
		}

		return tester;
	}

	protected describe(
		name: string,
		tester: ITester,
		testConfig: TTestConfig<T>
	) {
		beforeAll(async () => {
			await tester.prepare();
		});

		for (let index = 0; index < tester.total; index++) {
			const description =
				typeof this._options.description === "function"
					? this._options.description(name, index)
					: `step ${index ? `[${index}]` : ""} should pass`;
			let bailout = false;
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

	protected createEnv(testConfig: TTestConfig<T>): ITestEnv {
		if (typeof this._options.runner === "function" && !testConfig.noTest) {
			return createLazyTestEnv(10000);
		} else {
			return {
				expect,
				it,
				beforeEach,
				afterEach,
				jest
			};
		}
	}

	protected clean(folders: string[]) {
		for (const f of folders) {
			rimraf.sync(f);
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
}
