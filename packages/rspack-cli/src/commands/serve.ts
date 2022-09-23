import { createServer } from "@rspack/dev-server";
import { RspackCLI } from "../rspack-cli";
import { RspackCommand } from "../types";
import { commonOptions } from "../utils/options";

export class ServeCommand implements RspackCommand {
	async apply(cli: RspackCLI): Promise<void> {
		cli.program.command(
			"serve [entry..]",
			"serve",
			commonOptions,
			async options => {
				const config = await cli.loadConfig(options);

				const server = await createServer(config);
				await server.start();
			}
		);
	}
}
