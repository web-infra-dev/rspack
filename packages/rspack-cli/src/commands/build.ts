import { rspack } from "@rspack/core";
import util from "util";
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
			}
		);
	}
}
