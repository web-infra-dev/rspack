/** @type {import("@rspack/core").LoaderDefinition} */
export default function loader(content) {
	return content + `.using-loader { color: red; }`;
};
