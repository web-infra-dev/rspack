/** @type {import("@rspack/core").LoaderDefinition<{ value: string }>} */
export default function (source) {
	if (!this.data.pitched) {
		throw new Error("The loader pitch function did not run");
	}
	const current = this.loaders[this.loaderIndex];
	if (!current.request.includes("transform-loader.mjs")) {
		throw new Error(`Unexpected loader request: ${current.request}`);
	}
	if (current.request.includes("builtin:code-filter-loader")) {
		throw new Error(`The code filter leaked into the loader request: ${current.request}`);
	}
	for (const request of [
		this.request,
		this.remainingRequest,
		this.previousRequest,
		...this.loaders.map(loader => loader.request),
	]) {
		if (request.includes("|loader-filter:")) {
			throw new Error(`The filter identity leaked into the loader request: ${request}`);
		}
		if (request.includes("builtin:code-filter-loader")) {
			throw new Error(`The code filter leaked into loader context: ${request}`);
		}
	}
	const options = this.getOptions();
	if (Object.keys(options).join(",") !== "value") {
		throw new Error(`Unexpected loader options: ${JSON.stringify(options)}`);
	}
	const { value } = options;
	return `export default ${JSON.stringify(value)};`;
}

export const pitch = function (_remainingRequest, _previousRequest, data) {
	data.pitched = true;
};
