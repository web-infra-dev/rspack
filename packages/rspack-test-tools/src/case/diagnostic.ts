import assert from "node:assert";
import path from "node:path";
import merge from "webpack-merge";
import { readConfigFile } from "../helper";
import { normalizePlaceholder } from "../helper/expect/placeholder";
import { BasicCaseCreator } from "../test/creator";
import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerOptions
} from "../type";
import { getCompiler } from "./common";

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	steps: ({ name }) => [
		{
			config: async (context: ITestContext) => {
				const compiler = getCompiler(context, name);
				let options = defaultOptions(context);
				const custom = readConfigFile<ECompilerType.Rspack>(
					["rspack.config.js", "webpack.config.js"].map(i =>
						context.getSource(i)
					)
				)[0];
				if (custom) {
					options = merge(options, custom);
				}
				if (!global.printLogger) {
					options.infrastructureLogging = {
						level: "error"
					};
				}
				compiler.setOptions(options);
			},
			compiler: async (context: ITestContext) => {
				const compiler = getCompiler(context, name);
				compiler.createCompiler();
			},
			build: async (context: ITestContext) => {
				const compiler = getCompiler(context, name);
				await compiler.build();
			},
			run: async (env: ITestEnv, context: ITestContext) => {
				// no need to run, just check the snapshot of diagnostics
			},
			check: async (env: ITestEnv, context: ITestContext) => {
				await check(env, context, name, {
					snapshot: "./stats.err",
					snapshotErrors: "./raw-error.err",
					snapshotWarning: "./raw-warning.err",
					format: (output: string) => {
						// TODO: change to stats.errorStack
						// TODO: add `errorStack: false`
						return output.replace(/(│.* at ).*/g, "$1xxx");
					}
				});
			}
		}
	],
	concurrent: true
});

export function createDiagnosticCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
export type TDiagnosticOptions = {
	snapshot: string;
	snapshotErrors: string;
	snapshotWarning: string;
	format?: (output: string) => string;
};

function defaultOptions<T extends ECompilerType.Rspack>(
	context: ITestContext
): TCompilerOptions<T> {
	return {
		target: "node",
		context: context.getSource(),
		entry: {
			main: "./"
		},
		mode: "development",
		devServer: {
			hot: false
		},
		infrastructureLogging: {
			debug: false
		},
		output: {
			path: context.getDist()
		},
		experiments: {
			css: true,
			rspackFuture: {
				bundlerInfo: {
					force: false
				}
			},
			inlineConst: true,
			lazyBarrel: true
		}
	} as TCompilerOptions<T>;
}

async function check(
	env: ITestEnv,
	context: ITestContext,
	name: string,
	options: TDiagnosticOptions
) {
	const compiler = getCompiler(context, name);
	const stats = compiler.getStats();
	if (!stats) {
		throw new Error("Stats should exists");
	}
	assert(stats.hasErrors() || stats.hasWarnings());
	let output = normalizePlaceholder(
		stats.toString({
			all: false,
			errors: true,
			warnings: true
		})
	).replaceAll("\\", "/");

	const statsJson = stats.toJson({
		all: false,
		errors: true,
		warnings: true
	});
	const errors = (statsJson.errors || []).map(e => {
		// @ts-expect-error error message is already serialized in `stats.err`
		delete e.message;
		delete e.stack;
		return e;
	});
	const warnings = (statsJson.warnings || []).map(e => {
		// @ts-expect-error error message is already serialized in `stats.err`
		delete e.message;
		delete e.stack;
		return e;
	});

	if (typeof options.format === "function") {
		output = options.format(output);
	}

	env.expect.addSnapshotSerializer({
		test(received) {
			return typeof received === "string";
		},
		serialize(received) {
			return normalizePlaceholder((received as string).trim());
		}
	});

	const errorOutputPath = path.resolve(context.getSource(options.snapshot));
	const errorStatsOutputPath = path.resolve(
		context.getSource(options.snapshotErrors)
	);
	const warningStatsOutputPath = path.resolve(
		context.getSource(options.snapshotWarning)
	);
	env.expect(output).toMatchFileSnapshot(errorOutputPath);
	env.expect(errors).toMatchFileSnapshot(errorStatsOutputPath);
	env.expect(warnings).toMatchFileSnapshot(warningStatsOutputPath);
}
