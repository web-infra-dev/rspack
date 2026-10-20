export const raw = true;

export default function (source) {
	const filtered = this.loaders[this.loaderIndex + 1];
	if (!filtered?.loaderItem.data.pitched) {
		throw new Error("The filtered loader pitch function did not run");
	}
	if (!filtered.request.includes("transform-loader.mjs")) {
		throw new Error(`Unexpected filtered loader request: ${filtered.request}`);
	}
	return source;
}
