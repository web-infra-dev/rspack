import {
	ECompilerType,
	ITestContext,
	ITestProcessor,
	ITester,
	TRunnerFactory,
	TTestConfig
} from "../type";
import fs from "fs";
import path from "path";
import rimraf from "rimraf";
import { Tester } from "./tester";
import createLazyTestEnv from "../helper/legacy/createLazyTestEnv";

export interface IBasicCaseCreatorOptions<T extends ECompilerType> {
	clean?: boolean;
	describe?: boolean;
	timeout?: number;
	steps: (
		creatorConfig: IBasicCaseCreatorOptions<T> & {
			name: string;
			src: string;
			dist: string;
			temp: string | void;
		}
	) => ITestProcessor[];
	description?: (name: string) => string;
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
	}

	protected describe(
		name: string,
		tester: ITester,
		testConfig: TTestConfig<T>
	) {
		beforeAll(async () => {
			await tester.prepare();
		});
		const description =
			typeof this._options.description === "function"
				? this._options.description(name)
				: `${name} should compile`;
		it(
			description,
			async () => {
				await tester.compile();
				await tester.check(env);
			},
			this._options.timeout || 30000
		);

		afterAll(async () => {
			await tester.resume();
		});

		const env = this.createEnv(testConfig);
	}

	protected createEnv(testConfig: TTestConfig<T>) {
		if (typeof this._options.runner === "function" && !testConfig.noTest) {
			return createLazyTestEnv(10000);
		} else {
			return {
				it,
				beforeEach,
				afterEach
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
