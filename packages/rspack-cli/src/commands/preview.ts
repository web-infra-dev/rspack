import type { RspackCLI } from "../rspack-cli";
import { RspackDevServer } from "@rspack/dev-server";
import { RspackCommand, RspackPreviewCLIOptions } from "../types";
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
			["preview [dir]", "preview", "p"],
			"run the rspack server for build output",
			previewOptions,
			async options => {
				// config、configName are necessary for loadConfig
				const rspackOptions = {
					config: options.config,
					configName: options.configName,
					argv: {
						...options
					}
				};

				let config = await cli.loadConfig(rspackOptions);
				config = await getPreviewConfig(config, options);
				if (!Array.isArray(config)) {
					config = [config as RspackOptions];
				}

				config = config as MultiRspackOptions;

				// find the possible devServer config
				config = config.find(item => item.devServer) || config[0];

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

// get the devServerOptions from the config
async function getPreviewConfig(
	item: RspackOptions | MultiRspackOptions,
	options: RspackPreviewCLIOptions
): Promise<RspackOptions | MultiRspackOptions> {
	const internalPreviewConfig = async (item: RspackOptions) => {
		if (!item.devServer) {
			item.devServer = {};
		}
		// all of the options that a preview static server needs(maybe not all)
		item.devServer = {
			static: {
				directory: options.dir
					? transformPath(options.dir)
					: item.output?.path ?? transformPath(defaultRoot),
				publicPath: options.publicPath || "/"
			},
			port: options.port || 8080,
			host: options.host || item.devServer.host,
			open: options.open || item.devServer.open,
			server: options.server || item.devServer.server,
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

// transform dir to absolute path
function transformPath(dir: string) {
	return path.resolve(process.cwd(), dir);
}
