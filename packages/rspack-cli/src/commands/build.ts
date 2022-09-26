import { build } from "@rspack/core";
import { RspackCLI } from "../rspack-cli";
import { RspackCommand } from "../types";
import { commonOptions } from "../utils/options";
export class BuildCommand implements RspackCommand {
	async apply(cli: RspackCLI): Promise<void> {
		cli.program.command(
			["build [entry..]", "$0", "bundle", "b"],
			"run the rspack build",
			commonOptions,
			async options => {
				console.time("build");
				const compiler = await cli.createCompiler(options);
				const stats = await compiler.build();
				console.timeEnd("build");
			}
		);
	}
}
