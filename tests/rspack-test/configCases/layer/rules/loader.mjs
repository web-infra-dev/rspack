/** @type {import("@rspack/core").LoaderDefinition<{ value: any }>} */
export default function (source) {
	const options = this.getOptions();
	return `${source}
;
export const __loaderValue = ${JSON.stringify(options.value)};`;
};
