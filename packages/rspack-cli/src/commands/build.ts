import util from "util";
import type { RspackCLI } from "../rspack-cli";
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
				const stats = await util.promisify(compiler.build.bind(compiler))();
				if (stats.errors.length > 0) {
					throw new Error(stats.errors.map(x => x.message).join("\n"));
				}
				console.timeEnd("build");
			}
		);
	}
}
