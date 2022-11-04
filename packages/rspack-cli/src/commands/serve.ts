import type { RspackCLI } from "../rspack-cli";
import { RspackDevServer } from "@rspack/dev-server";
import { RspackCommand } from "../types";
import { commonOptions } from "../utils/options";
import { createCompiler } from "@rspack/core";
export class ServeCommand implements RspackCommand {
	async apply(cli: RspackCLI): Promise<void> {
		cli.program.command(
			["serve [entry..]", "server", "s"],
			"run the rspack dev server.",
			commonOptions,
			async options => {
				const compiler = await cli.createCompiler(options);
				const server = new RspackDevServer(compiler);
				await server.start();
			}
		);
	}
}
