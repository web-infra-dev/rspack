/** @type {import("@rspack/core").LoaderDefinition<{ f(): any }>} */
export default function (source) {
	return `${source}, World!`;
};
