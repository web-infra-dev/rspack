import { build } from "@rspack/core";
import { RspackCLI } from "../rspack-cli";
import { RspackCommand } from "../types";
import { commonOptions } from "../utils/options";
export class BuildCommand implements RspackCommand {
	async apply(cli: RspackCLI): Promise<void> {
		cli.program.command(
			["build [entry..]", "$0"],
			"build",
			commonOptions,
			async options => {
				const config = await cli.loadConfig(options);
				console.time("build");
				console.log({ config });
				const stats = await build(config);
				console.timeEnd("build");
			}
		);
	}
}
