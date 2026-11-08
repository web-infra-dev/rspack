/** @type {import("@rspack/core").LoaderDefinition<{ value: string }>} */
export default function (source) {
	if (!this.data.pitched) {
		throw new Error("The parallel loader pitch function did not run");
	}
	return source.replace('"__VALUE__"', JSON.stringify(this.getOptions().value));
}

export const pitch = function (_remainingRequest, _previousRequest, data) {
	data.pitched = true;
};
