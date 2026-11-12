export default function (source) {
	const filtered = this.loaders[this.loaderIndex + 1];
	if (!filtered?.loaderItem.data.pitched) {
		throw new Error("The filtered parallel loader pitch function did not run");
	}
	return source;
}
