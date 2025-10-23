import fs from "node:fs";
import type {
	MultiStats,
	MultiStatsOptions,
	Stats,
	StatsOptions
} from "@rspack/core";
import type { RspackCLI } from "../cli";
import type { RspackCommand } from "../types";
import {
	type CommonOptionsForBuildAndServe,
	commonOptions,
	commonOptionsForBuildAndServe,
	normalizeCommonOptions,
	setDefaultNodeEnv
} from "../utils/options";

type BuildOptions = CommonOptionsForBuildAndServe & {
	json?: boolean | string;
};

async function runBuild(cli: RspackCLI, options: BuildOptions): Promise<void> {
	setDefaultNodeEnv(options, "production");
	normalizeCommonOptions(options, "build");

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

		if (stats?.hasErrors()) {
			process.exitCode = 1;
		}

		if (!compiler || !stats) {
			return;
		}

		const getStatsOptions = () => {
			if (cli.isMultipleCompiler(compiler)) {
				return {
					children: compiler.compilers.map(item =>
						item.options ? item.options.stats : undefined
					)
				} satisfies MultiStatsOptions;
			}
			return compiler.options?.stats;
		};

		const statsOptions = getStatsOptions() as StatsOptions;

		if (options.json && createJsonStringifyStream) {
			const handleWriteError = (error: Error) => {
				logger.error(error);
				process.exit(2);
			};
			if (options.json === true) {
				createJsonStringifyStream(stats.toJson(statsOptions))
					.on("error", handleWriteError)
					.pipe(process.stdout)
					.on("error", handleWriteError)
					.on("close", () => process.stdout.write("\n"));
			} else if (typeof options.json === "string") {
				createJsonStringifyStream(stats.toJson(statsOptions))
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

	const compiler = await cli.createCompiler(options, "build", errorHandler);

	if (!compiler || cli.isWatch(compiler)) {
		return;
	}

	compiler.run((error: Error | null, stats: Stats | MultiStats | undefined) => {
		compiler.close(closeErr => {
			if (closeErr) {
				logger.error(closeErr);
			}
			errorHandler(error, stats);
		});
	});
}

export class BuildCommand implements RspackCommand {
	async apply(cli: RspackCLI): Promise<void> {
		const command = cli.program
			.command("", "run the Rspack build")
			.alias("build")
			.alias("bundle")
			.alias("b");

		commonOptionsForBuildAndServe(commonOptions(command)).option(
			"--json [path]",
			"emit stats json"
		);

		command.action(async (options: BuildOptions) => {
			await runBuild(cli, options);
		});
	}
}
