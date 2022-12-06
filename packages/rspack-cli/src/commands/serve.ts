import type { RspackCLI } from "../rspack-cli";
import { RspackDevServer } from "@rspack/dev-server";
import { RspackCommand } from "../types";
import { commonOptions } from "../utils/options";
export class ServeCommand implements RspackCommand {
	async apply(cli: RspackCLI): Promise<void> {
		cli.program.command(
			["serve [entry..]", "server", "s"],
			"run the rspack dev server.",
			commonOptions,
			async options => {
				const rspackOptions = { ...options };
				// Todo will support more complex options in the future
				rspackOptions.argv = options;
				const compiler = await cli.createCompiler(rspackOptions, "development");
				const server = new RspackDevServer(compiler);
				await server.start();
			}
		);
	}
}
