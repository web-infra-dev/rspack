import { RspackCLI } from "./rspack-cli";

export async function runCLI(argv: string[]) {
	const cli = new RspackCLI();
	await cli.run(argv);
}
