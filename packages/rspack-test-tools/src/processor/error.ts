import fs from "fs";
import path from "path";
import { StatsError, StatsWarnings } from "@rspack/core";
import prettyFormat from "pretty-format";
import merge from "webpack-merge";

import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompiler,
	TCompilerOptions
} from "../type";
import { SimpleTaskProcessor } from "./simple";

type TStatsDiagnostics = {
	errors: StatsError[];
	warnings: StatsWarnings[];
};

const CWD_PATTERN = new RegExp(
	path.join(process.cwd(), "../../").replace(/\\/g, "/"),
	"gm"
);
const ERROR_STACK_PATTERN = /(?:\n\s+at\s.*)+/gm;

function cleanError(err: Error) {
	const result: Partial<Record<keyof Error, any>> = {};
	for (const key of Object.getOwnPropertyNames(err)) {
		result[key as keyof Error] = err[key as keyof Error];
	}

	if (result.message) {
		result.message = err.message.replace(ERROR_STACK_PATTERN, "");
	}

	if (result.stack) {
		result.stack = result.stack.replace(ERROR_STACK_PATTERN, "");
	}

	return result;
}

function serialize(received: unknown) {
	return prettyFormat(received, prettyFormatOptions)
		.replace(CWD_PATTERN, "<cwd>")
		.trim();
}

const prettyFormatOptions = {
	escapeRegex: false,
	printFunctionName: false,
	plugins: [
		{
			test(val: any) {
				return typeof val === "string";
			},
			print(val: any) {
				return `"${val
					.replace(/\\/gm, "/")
					.replace(/"/gm, '\\"')
					.replace(/\r?\n/gm, "\\n")}"`;
			}
		}
	]
};

export interface IErrorProcessorOptions<T extends ECompilerType> {
	name: string;
	compilerType: T;
	options?: (
		options: TCompilerOptions<T>,
		context: ITestContext
	) => TCompilerOptions<T>;
	build?: (context: ITestContext, compiler: TCompiler<T>) => Promise<void>;
	check?: (stats: TStatsDiagnostics) => Promise<void>;
}

export class ErrorProcessor<
	T extends ECompilerType
> extends SimpleTaskProcessor<T> {
	constructor(protected _errorOptions: IErrorProcessorOptions<T>) {
		super({
			options: (context: ITestContext): TCompilerOptions<T> => {
				let options = {
					context: path.resolve(__dirname, "../../tests/fixtures/errors"),
					mode: "none",
					devtool: false,
					optimization: {
						minimize: false,
						moduleIds: "named",
						chunkIds: "named"
					},
					experiments: {
						css: true,
						rspackFuture: {
							bundlerInfo: {
								force: false
							}
						}
					}
				} as TCompilerOptions<T>;
				if (typeof _errorOptions.options === "function") {
					options = merge(options, _errorOptions.options(options, context));
				}
				if (options.mode === "production") {
					if (options.optimization) options.optimization.minimize = true;
					else options.optimization = { minimize: true };
				}
				return options;
			},
			build: _errorOptions.build,
			compilerType: _errorOptions.compilerType,
			name: _errorOptions.name
		});
	}

	async compiler(context: ITestContext) {
		await super.compiler(context);
		const compiler = this.getCompiler(context).getCompiler();
		if (compiler) {
			compiler.outputFileSystem = {
				// CHANGE: rspack outputFileSystem `mkdirp` uses option `{ recursive: true }`, webpack's second parameter is alway a callback
				mkdir(
					dir: string,
					maybeOptionOrCallback: unknown,
					maybeCallback: unknown
				) {
					if (typeof maybeOptionOrCallback === "function") {
						maybeOptionOrCallback();
					} else if (typeof maybeCallback === "function") {
						maybeCallback();
					}
				},
				writeFile(file: string, content: string, callback: () => {}) {
					callback();
				},
				stat(file: string, callback: (e: Error) => {}) {
					callback(new Error("ENOENT"));
				},
				mkdirSync() {},
				writeFileSync() {}
			} as unknown as typeof fs;
		}
	}
	async run(env: ITestEnv, context: ITestContext) {
		// do nothing
	}

	async check(env: ITestEnv, context: ITestContext) {
		const compiler = this.getCompiler(context);
		const stats = compiler.getStats();
		env.expect(typeof stats).toBe("object");
		const statsResult = stats!.toJson({ errorDetails: false });
		env.expect(typeof statsResult).toBe("object");
		const { errors, warnings } = statsResult;
		env.expect(Array.isArray(errors)).toBe(true);
		env.expect(Array.isArray(warnings)).toBe(true);

		await this._errorOptions.check?.({
			errors: errors as StatsError[],
			warnings: warnings as StatsWarnings[]
		});
	}

	static addSnapshotSerializer(expectImpl: jest.Expect) {
		expectImpl.addSnapshotSerializer({
			test(received) {
				return received.errors || received.warnings;
			},
			print(received) {
				return serialize({
					errors: (received as TStatsDiagnostics).errors.map(e =>
						cleanError(e as unknown as Error)
					),
					warnings: (received as TStatsDiagnostics).warnings.map(e =>
						cleanError(e as unknown as Error)
					)
				});
			}
		});

		expectImpl.addSnapshotSerializer({
			test(received) {
				return received.message;
			},
			print(received) {
				return serialize(cleanError(received as Error));
			}
		});
	}
}
