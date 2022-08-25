import { Rspack } from ".";
export async function build(config: any) {
	const rspack = new Rspack(config);
	const stats = await rspack.build();
	if (stats.errors.length > 0) {
		throw new Error(stats.errors[0].message);
	}
}
