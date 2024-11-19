import { stripVTControlCharacters as stripAnsi } from "node:util";
import { diff as jestDiff } from "jest-diff";

import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerOptions
} from "../type";
import { SimpleTaskProcessor } from "./simple";

const CURRENT_CWD = process.cwd();

class RspackTestDiff {
	constructor(public value: string) {}
}

export interface IDefaultsConfigProcessorOptions<T extends ECompilerType> {
	options?: (context: ITestContext) => TCompilerOptions<T>;
	cwd?: string;
	name: string;
	diff: (
		diff: jest.JestMatchers<RspackTestDiff>,
		defaults: jest.JestMatchers<TCompilerOptions<T>>
	) => Promise<void>;
	compilerType: T;
}

export class DefaultsConfigProcessor<
	T extends ECompilerType
> extends SimpleTaskProcessor<T> {
	private defaultConfig: TCompilerOptions<T>;

	constructor(
		protected _defaultsConfigOptions: IDefaultsConfigProcessorOptions<T>
	) {
		super({
			options: context => {
				let res: TCompilerOptions<T>;
				if (typeof _defaultsConfigOptions.options === "function") {
					res = _defaultsConfigOptions.options(context);
				} else {
					res = {};
				}
				if (!("mode" in res)) {
					res.mode = "none";
				}
				return res;
			},
			compilerType: _defaultsConfigOptions.compilerType,
			name: _defaultsConfigOptions.name
		});
		this.defaultConfig = DefaultsConfigProcessor.getDefaultConfig(CURRENT_CWD, {
			mode: "none"
		}) as TCompilerOptions<T>;
	}

	async compiler(context: ITestContext) {}
	async build(context: ITestContext) {}
	async run(env: ITestEnv, context: ITestContext) {}

	async check(env: ITestEnv, context: ITestContext) {
		const compiler = this.getCompiler(context);
		const config = DefaultsConfigProcessor.getDefaultConfig(
			this._defaultsConfigOptions.cwd || CURRENT_CWD,
			compiler.getOptions()
		);
		const diff = stripAnsi(
			jestDiff(this.defaultConfig, config, { expand: false, contextLines: 0 })!
		);
		await this._defaultsConfigOptions.diff(
			env.expect(new RspackTestDiff(diff)),
			env.expect(this.defaultConfig)
		);
	}

	async before(context: ITestContext): Promise<void> {}
	async after(context: ITestContext): Promise<void> {}
	async beforeAll(context: ITestContext): Promise<void> {}
	async afterAll(context: ITestContext) {}

	protected getCompiler(context: ITestContext) {
		return context.getCompiler(this._options.name, this._options.compilerType);
	}

	static getDefaultConfig(
		cwd: string,
		config: TCompilerOptions<ECompilerType>
	): TCompilerOptions<ECompilerType> {
		process.chdir(cwd);
		const { applyWebpackOptionsDefaults, getNormalizedWebpackOptions } =
			require("@rspack/core").config;
		const normalizedConfig = getNormalizedWebpackOptions(config);
		applyWebpackOptionsDefaults(normalizedConfig);
		// make snapshot stable
		(normalizedConfig as any).experiments.rspackFuture.bundlerInfo.version =
			"$version$";
		process.chdir(CURRENT_CWD);
		return normalizedConfig;
	}
}
