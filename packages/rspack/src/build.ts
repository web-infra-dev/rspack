import { Rspack } from ".";
export function rspack(config: any) {
	const rspack = new Rspack(config);
	return rspack;
}
export async function build(config: any) {
	const compiler = rspack(config);
	const stats = await compiler.build();
	if (stats.errors.length > 0) {
		throw new Error(stats.errors[0].message);
	}
}
