export async function load(): Promise<string> {
	const { c } = await import(
		/* webpackChunkName: "line-after-await" */
		// eslint-disable-next-line no-restricted-imports
		"./chunk?line-after-await"
	);
	return c;
}
