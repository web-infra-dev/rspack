import * as fs from "fs";
import type { RspackCLI } from "../rspack-cli";
import { RspackCommand } from "../types";
import { commonOptions } from "../utils/options";
import { Stats } from "@rspack/core/src/stats";
import { Compiler, MultiCompiler } from "@rspack/core";
import MultiStats from "@rspack/core/src/multiStats";

export class BuildCommand implements RspackCommand {
	async apply(cli: RspackCLI): Promise<void> {
		cli.program.command(
			["build [entry..]", "$0", "bundle", "b"],
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
				let createJsonStringifyStream;
				if (options.json) {
					const jsonExt = await import("@discoveryjs/json-ext");
					createJsonStringifyStream = jsonExt.stringifyStream;
				}

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
						: compiler.options
						? compiler.options.stats
						: undefined;
					if (options.json && createJsonStringifyStream) {
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

				let rspackOptions = { ...options, argv: { ...options } };

				const errorHandler = (err, Stats) => {
					callback(err, Stats);
					if (!cli.isWatch(compiler)) console.timeEnd("build");
				};

				const compiler = await cli.createCompiler(rspackOptions, errorHandler);
			}
		);
	}
}
