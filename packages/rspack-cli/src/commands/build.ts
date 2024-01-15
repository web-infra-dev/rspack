import * as fs from "fs";
import type { RspackCLI } from "../rspack-cli";
import { RspackCommand } from "../types";
import { commonOptions, setBuiltinEnvArg } from "../utils/options";
import { MultiStats, Stats } from "@rspack/core";

export class BuildCommand implements RspackCommand {
	async apply(cli: RspackCLI): Promise<void> {
		cli.program.command(
			["build", "$0", "bundle", "b"],
			"run the rspack build",
			yargs =>
				commonOptions(yargs).options({
					analyze: {
						type: "boolean",
						default: false,
						describe: "analyze"
					},
					json: {
						describe: "emit stats json"
					}
				}),
			async options => {
				if (options.watch) {
					// @ts-expect-error
					setBuiltinEnvArg(options.env, "WATCH", true);
				} else {
					// @ts-expect-error
					setBuiltinEnvArg(options.env, "BUNDLE", true);
					// @ts-expect-error
					setBuiltinEnvArg(options.env, "BUILD", true);
				}
				const logger = cli.getLogger();
				let createJsonStringifyStream: typeof import("@discoveryjs/json-ext").stringifyStream;
				if (options.json) {
					const jsonExt = await import("@discoveryjs/json-ext");
					createJsonStringifyStream = jsonExt.default.stringifyStream;
				}

				const errorHandler = (
					error: Error | null,
					stats: Stats | MultiStats | undefined
				) => {
					if (error) {
						logger.error(error);
						process.exit(2);
					}
					if (stats && stats.hasErrors()) {
						process.exitCode = 1;
					}
					if (!compiler || !stats) {
						return;
					}
					const statsOptions = cli.isMultipleCompiler(compiler)
						? {
								children: compiler.compilers.map(compiler =>
									compiler.options ? compiler.options.stats : undefined
								)
						  }
						: compiler.options
						? compiler.options.stats
						: undefined;
					if (options.json && createJsonStringifyStream) {
						const handleWriteError = (error: Error) => {
							logger.error(error);
							process.exit(2);
						};
						if (options.json === true) {
							createJsonStringifyStream(stats.toJson(statsOptions as any))
								.on("error", handleWriteError)
								.pipe(process.stdout)
								.on("error", handleWriteError)
								.on("close", () => process.stdout.write("\n"));
						} else if (typeof options.json === "string") {
							createJsonStringifyStream(stats.toJson(statsOptions as any))
								.on("error", handleWriteError)
								.pipe(fs.createWriteStream(options.json))
								.on("error", handleWriteError)
								// Use stderr to logging
								.on("close", () => {
									process.stderr.write(
										`[rspack-cli] ${cli.colors.green(
											`stats are successfully stored as json to ${options.json}`
										)}\n`
									);
								});
						}
					} else {
						const printedStats = stats.toString(statsOptions);
						// Avoid extra empty line when `stats: 'none'`
						if (printedStats) {
							logger.raw(printedStats);
						}
					}
				};

				const rspackOptions = { ...options, argv: { ...options } };

				const compiler = await cli.createCompiler(
					rspackOptions,
					"build",
					errorHandler
				);

				if (!compiler) return;
				if (cli.isWatch(compiler)) {
					return;
				} else {
					compiler.run(errorHandler);
				}
			}
		);
	}
}
