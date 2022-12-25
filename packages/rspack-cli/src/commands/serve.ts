import type { RspackCLI } from "../rspack-cli";
import { RspackDevServer } from "@rspack/dev-server";
import { RspackCommand } from "../types";
import { commonOptions, normalizeEnv } from "../utils/options";
export class ServeCommand implements RspackCommand {
	async apply(cli: RspackCLI): Promise<void> {
		cli.program.command(
			["serve [entry..]", "server", "s"],
			"run the rspack dev server.",
			commonOptions,
			async options => {
				const env = normalizeEnv(options);
				const rspackOptions = {
					...options,
					env,
					argv: {
						...options,
						env
					}
				};
				const compiler = await cli.createCompiler(rspackOptions, "development");
				const server = new RspackDevServer(compiler);
				await server.start();
			}
		);
	}
}
