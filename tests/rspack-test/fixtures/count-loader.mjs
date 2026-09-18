let counter = 0;

/** @type {import("@rspack/core").LoaderDefinition} */
export default function () {
	return `export default ${counter++};`;
};
