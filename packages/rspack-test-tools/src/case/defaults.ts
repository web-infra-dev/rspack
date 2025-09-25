import path from "node:path";
import { stripVTControlCharacters as stripAnsi } from "node:util";
import { diff as jestDiff } from "jest-diff";
import { BasicCaseCreator } from "../test/creator";
import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerOptions
} from "../type";
import { getCompiler } from "./common";

const CURRENT_CWD = process.cwd();

const creator = new BasicCaseCreator({
	clean: true,
	steps: ({ name, caseConfig: _caseConfig }) => {
		const caseConfig = _caseConfig as TDefaultsCaseConfig;
		return [
			{
				config: async (context: ITestContext) => {
					const compiler = getCompiler(context, name);
					compiler.setOptions(options(context, caseConfig.options));
				},
				compiler: async (context: ITestContext) => {
					// no need to create compiler, just check the snapshot of compiler options
				},
				build: async (context: ITestContext) => {
					// no need to build, just check the snapshot of compiler options
				},
				run: async (env: ITestEnv, context: ITestContext) => {
					// no need to run, just check the snapshot of compiler options
				},
				check: async (env: ITestEnv, context: ITestContext) => {
					await check(env, context, name, caseConfig);
				}
			}
		];
	},
	concurrent: false
});

const srcDir = path.resolve(
	__dirname,
	"../../../../tests/rspack-test/fixtures"
);
const distDir = path.resolve(
	__dirname,
	"../../../../tests/rspack-test/js/defaults"
);

export function defineDefaultsCase(
	name: string,
	caseConfig: TDefaultsCaseConfig
) {
	creator.create(name, srcDir, path.join(distDir, name), undefined, {
		caseConfig,
		description: () => caseConfig.description
	});
}

export function getRspackDefaultConfig(
	context: string,
	config: TCompilerOptions<ECompilerType>
): TCompilerOptions<ECompilerType> {
	process.chdir(context);
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

export type TDefaultsCaseConfig = {
	options?: (context: ITestContext) => TCompilerOptions<ECompilerType.Rspack>;
	cwd?: string;
	diff: (
		diff: jest.JestMatchers<RspackTestDiff>,
		defaults: jest.JestMatchers<TCompilerOptions<ECompilerType.Rspack>>
	) => Promise<void>;
	description: string;
};

function options(
	context: ITestContext,
	custom?: (context: ITestContext) => TCompilerOptions<ECompilerType.Rspack>
) {
	let res: TCompilerOptions<ECompilerType.Rspack>;
	if (typeof custom === "function") {
		res = custom(context);
	} else {
		res = {};
	}
	if (!("mode" in res)) {
		res.mode = "none";
	}
	if (!("context" in res)) {
		res.context = context.getSource();
	}
	return res;
}

class RspackTestDiff {
	constructor(public value: string) {}
}

async function check(
	env: ITestEnv,
	context: ITestContext,
	name: string,
	options: {
		cwd?: string;
		diff: (
			diff: jest.JestMatchers<RspackTestDiff>,
			defaults: jest.JestMatchers<TCompilerOptions<ECompilerType.Rspack>>
		) => Promise<void>;
	}
) {
	const compiler = getCompiler(context, name);
	const config = getRspackDefaultConfig(
		context.getSource(),
		compiler.getOptions()
	);
	const defaultConfig = getRspackDefaultConfig(context.getSource(), {
		mode: "none"
	});
	const diff = stripAnsi(
		jestDiff(defaultConfig, config, { expand: false, contextLines: 0 })!
	);
	await options.diff(
		env.expect(new RspackTestDiff(diff)),
		env.expect(defaultConfig)
	);
}
