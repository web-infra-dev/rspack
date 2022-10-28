import { rspack } from "@rspack/core";
import * as util from "util";
import * as fs from 'fs';
import type { RspackCLI } from "../rspack-cli";
import { RspackCommand } from "../types";
import { commonOptions } from "../utils/options";

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
						describe: "emit stats json",
					}
				}),
			async options => {
				const config = await cli.loadConfig(options);
				if (options.analyze) {
					const { BundleAnalyzerPlugin } = await import(
						"webpack-bundle-analyzer"
					);
					(config.plugins ??= []).push({
						name: "rspack-bundle-analyzer",
						apply(compiler) {
							new BundleAnalyzerPlugin({
								generateStatsFile: true,
								// TODO: delete this once runtime refacted.
								excludeAssets: "runtime.js"
							}).apply(compiler as any);
						}
					});
				}
				console.time("build");
				const stats = await util.promisify(rspack)(config);
				const statsJson = stats.toJson();
				if (statsJson.errors.length > 0) {
					throw new Error(statsJson.errors.map(x => x.message).join("\n"));
				}
				console.timeEnd("build");

				if (options.json) {
					const { stringifyStream: createJsonStringifyStream } = await import("@discoveryjs/json-ext");
					const logger = cli.getLogger();
					const handleWriteError = (error) => {
						logger.error(error);
						process.exit(2);
					};
					if (options.json === true) {
						createJsonStringifyStream(statsJson)
							.on("error", handleWriteError)
							.pipe(process.stdout)
							.on("error", handleWriteError)
							.on("close", () => process.stdout.write("\n"));
					} else if (typeof options.json === 'string') {
						createJsonStringifyStream(statsJson)
							.on("error", handleWriteError)
							.pipe(fs.createWriteStream(options.json))
							.on("error", handleWriteError)
							// Use stderr to logging
							.on("close", () => {
								process.stderr.write(`[rspack-cli] ${cli.colors.green(`stats are successfully stored as json to ${options.json}`)}\n`);
							});
					}
				}
			}
		);
	}
}
