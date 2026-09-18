/** @type {import("@rspack/core").LoaderDefinition<{ value: any }>} */
export default function loader(content) {
	return content.replace(/test/, "NEW");
};
