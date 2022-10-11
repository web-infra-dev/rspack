import { build, Plugin } from "@rspack/core";
import { RspackCLI } from "../rspack-cli";
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
				let config = await cli.loadConfig(options);
				if (options.analyze) {
					const { BundleAnalyzerPlugin } = await import(
						"webpack-bundle-analyzer"
					);
					(config.plugins ??= []).push({
						name: "rspack-bundle-analyzer",
						apply(compiler) {
							new BundleAnalyzerPlugin({ generateStatsFile: true }).apply({
								...compiler,
								// hack for this.compiler in BundleAnalyzerPlugin
								outputPath: compiler.options.output.path,
								outputFileSystem: { constructor: undefined }
							} as any);
						}
					});
				}
				console.time("build");
				const compiler = await cli.createCompiler(config);
				const stats = await compiler.build();
				console.timeEnd("build");
			}
		);
	}
}
