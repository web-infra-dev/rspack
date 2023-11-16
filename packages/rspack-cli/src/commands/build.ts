import * as fs from "fs";
import type { RspackCLI } from "../rspack-cli";
import { RspackCommand } from "../types";
import { commonOptions } from "../utils/options";
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
				const logger = cli.getLogger();
				// @ts-expect-error
				let createJsonStringifyStream;
				if (options.json) {
					const jsonExt = await import("@discoveryjs/json-ext");
					createJsonStringifyStream = jsonExt.default.stringifyStream;
				}

				// @ts-expect-error
				const callback = (error, stats: Stats | MultiStats) => {
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
						: // @ts-expect-error
						compiler.options
						? // @ts-expect-error
						  compiler.options.stats
						: undefined;
					// @ts-expect-error
					if (options.json && createJsonStringifyStream) {
						// @ts-expect-error
						const handleWriteError = error => {
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

				// @ts-expect-error
				const errorHandler = (err, Stats) => {
					callback(err, Stats);
				};

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
