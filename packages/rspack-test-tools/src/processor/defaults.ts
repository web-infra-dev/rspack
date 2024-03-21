import { SimpleTaskProcessor } from "./simple";
import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerOptions
} from "../type";
import { diff as jestDiff } from "jest-diff";
import stripAnsi from "strip-ansi";
import path from "path";

const CASE_CWD = path.resolve(__dirname, "../../../rspack");
const CURRENT_CWD = process.cwd();

const quoteMeta = (str: string) => str.replace(/[-[\]\\/{}()*+?.^$|]/g, "\\$&");
const cwdRegExp = new RegExp(
	`${quoteMeta(CASE_CWD)}((?:\\\\)?(?:[a-zA-Z.\\-_]+\\\\)*)`,
	"g"
);
const escapedCwd = JSON.stringify(CASE_CWD).slice(1, -1);
const escapedCwdRegExp = new RegExp(
	`${quoteMeta(escapedCwd)}((?:\\\\\\\\)?(?:[a-zA-Z.\\-_]+\\\\\\\\)*)`,
	"g"
);
const normalize = (str: string) => {
	if (CASE_CWD.startsWith("/")) {
		str = str.replace(new RegExp(quoteMeta(CASE_CWD), "g"), "<cwd>");
	} else {
		str = str.replace(cwdRegExp, (m, g) => `<cwd>${g.replace(/\\/g, "/")}`);
		str = str.replace(
			escapedCwdRegExp,
			(m, g) => `<cwd>${g.replace(/\\\\/g, "/")}`
		);
	}
	str = str.replace(/@@ -\d+,\d+ \+\d+,\d+ @@/g, "@@ ... @@");
	return str;
};

class Diff {
	constructor(public value: string) {}
}

export interface IDefaultsConfigProcessorOptions {
	options?: (context: ITestContext) => TCompilerOptions<ECompilerType.Rspack>;
	cwd?: string;
	name: string;
	diff: (diff: any, defaults: any) => Promise<void>;
}

export class DefaultsConfigTaskProcessor extends SimpleTaskProcessor<ECompilerType.Rspack> {
	private defaultConfig: TCompilerOptions<ECompilerType.Rspack>;

	constructor(
		protected _defaultsConfigOptions: IDefaultsConfigProcessorOptions
	) {
		super({
			options: context => {
				let res;
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
			compilerType: ECompilerType.Rspack,
			name: _defaultsConfigOptions.name
		});
		this.defaultConfig = DefaultsConfigTaskProcessor.getDefaultConfig(
			CASE_CWD,
			{
				mode: "none"
			}
		);
	}

	async compiler(context: ITestContext) {
		throw new Error("Not support");
	}
	async build(context: ITestContext) {
		throw new Error("Not support");
	}
	async run(env: ITestEnv, context: ITestContext) {
		throw new Error("Not support");
	}

	async check(env: ITestEnv, context: ITestContext) {
		const compiler = this.getCompiler(context);
		const config = DefaultsConfigTaskProcessor.getDefaultConfig(
			this._defaultsConfigOptions.cwd || CASE_CWD,
			compiler.getOptions()
		);
		const diff = stripAnsi(
			jestDiff(this.defaultConfig, config, { expand: false, contextLines: 0 })!
		);
		await this._defaultsConfigOptions.diff(
			expect(new Diff(diff)),
			expect(this.defaultConfig)
		);
	}

	async before(context: ITestContext): Promise<void> {}
	async after(context: ITestContext): Promise<void> {}
	async beforeAll(context: ITestContext): Promise<void> {
		throw new Error("Not support");
	}
	async afterAll(context: ITestContext) {
		throw new Error("Not support");
	}

	protected getCompiler(context: ITestContext) {
		return context.getCompiler(this._options.name, this._options.compilerType);
	}

	static getDefaultConfig(
		cwd: string,
		config: TCompilerOptions<ECompilerType.Rspack>
	): TCompilerOptions<ECompilerType.Rspack> {
		process.chdir(cwd);
		const { applyWebpackOptionsDefaults, getNormalizedWebpackOptions } =
			require("@rspack/core").config;
		config = getNormalizedWebpackOptions(config);
		applyWebpackOptionsDefaults(config);
		// make snapshot stable
		(config as any).experiments.rspackFuture.bundlerInfo.version = "$version$";
		process.chdir(CURRENT_CWD);
		return config;
	}

	static addSnapshotSerializer() {
		expect.addSnapshotSerializer({
			test(value) {
				return value instanceof Diff;
			},
			print(received) {
				return normalize((received as Diff).value);
			}
		});

		expect.addSnapshotSerializer({
			test(value) {
				return typeof value === "string";
			},
			print(received) {
				return JSON.stringify(normalize(received as string));
			}
		});
	}
}
