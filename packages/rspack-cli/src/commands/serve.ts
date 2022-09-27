import { Rspack } from "@rspack/core";
import { RspackDevServer } from "@rspack/dev-server";
import { RspackCLI } from "../rspack-cli";
import { RspackCommand } from "../types";
import { commonOptions } from "../utils/options";

export class ServeCommand implements RspackCommand {
	async apply(cli: RspackCLI): Promise<void> {
		cli.program.command(
			["serve [entry..]", "server", "s"],
			"run the rspack dev server.",
			commonOptions,
			async options => {
				const config = await cli.loadConfig(options);
				const compiler = new Rspack(config);
				const server = new RspackDevServer(compiler);
				await server.start();
			}
		);
	}
}
