import type { RspackCLI } from "../rspack-cli";
import { RspackDevServer } from "@rspack/dev-server";
import {  RspackCommand } from "../types";
import {  previewOptions } from "../utils/options";
import { Compiler, DevServer, rspack, RspackOptions } from "@rspack/core";
import path from 'node:path';
import sirv from 'sirv';
import connect from 'connect'
export class PreviewCommand implements RspackCommand {
	async apply(cli: RspackCLI): Promise<void> {
		cli.program.command(
			["preview [root..]", "preview", "p"],
			"run the rspack server for build output",
			previewOptions,
			async options => {
                const rspackOptions = {
					...options,
					argv: {
						...options
					}
				};

                let config = await cli.loadConfig(rspackOptions);
                if(Array.isArray(config)) {
                    config = config[0]
                }
                config = config as RspackOptions;
                const devServerOption = {...config.devServer ?? {}, static: {
                    directory: path.resolve(process.cwd(), config.output?.path ?? 'dist'),
                    publicPath: config.output?.publicPath ?? '/',
                }};
                
                let compiler = rspack({entry: {}})
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
