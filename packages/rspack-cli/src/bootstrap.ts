import { RspackCLI } from "./rspack-cli";

export async function runCLI(argv: string[]) {
	const cli = new RspackCLI();
	try {
		await cli.run(argv);
	} catch (err) {
		console.error(err);
	}
}
