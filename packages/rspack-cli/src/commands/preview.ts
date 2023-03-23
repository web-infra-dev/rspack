import type { RspackCLI } from "../rspack-cli";
import { RspackDevServer } from "@rspack/dev-server";
import { RspackCommand, RspackCLIPreviewOptions } from "../types";
import { previewOptions } from "../utils/options";
import {
	DevServer,
	rspack,
	RspackOptions,
	MultiRspackOptions
} from "@rspack/core";
import path from "node:path";

const defaultRoot = "dist";
export class PreviewCommand implements RspackCommand {
	async apply(cli: RspackCLI): Promise<void> {
		cli.program.command(
			["preview [root]", "preview", "p"],
			"run the rspack server for build output",
			previewOptions,
			async options => {
				console.log(options);

				const rspackOptions = {
					config: options.config,
					configName: options.configName,
					argv: {
						...options
					}
				};

				let config = await cli.loadConfig(rspackOptions);
				config = await getPreviewConfig(config, options);
				if (Array.isArray(config)) {
					config = config[0];
				}
				config = config as RspackOptions;

				const devServerOption = config.devServer as DevServer;

				let compiler = rspack({ entry: {} });
				try {
					const server = new RspackDevServer(devServerOption, compiler);

					await server.start();
				} catch (error) {
					const logger = cli.getLogger();
					logger.error(error);

					process.exit(2);
				}
			}
		);
	}
}
async function getPreviewConfig(
	item: RspackOptions | MultiRspackOptions,
	options: RspackCLIPreviewOptions
): Promise<RspackOptions | MultiRspackOptions> {
	const internalPreviewConfig = async (item: RspackOptions) => {
		if (!item.devServer) {
			item.devServer = {};
		}
		item.devServer = {
			static: {
				directory: options.root
					? transformPath(options.root)
					: item.output.path || transformPath(defaultRoot),
				publicPath: options.publicPath || "/"
			},
			port: options.port || 8080,
			host: options.host || "localhost",
			open: options.open || false,
			server: options.server || "http",
			historyApiFallback: item.devServer.historyApiFallback
		};
		return item;
	};

	if (Array.isArray(item)) {
		return Promise.all(item.map(internalPreviewConfig));
	} else {
		return internalPreviewConfig(item as RspackOptions);
	}
}
function transformPath(dir: string) {
	return path.resolve(process.cwd(), dir);
}
