/** @type {import("@rspack/core").LoaderDefinition} */
export default function (content) {
	return (
		"module.exports = " +
		JSON.stringify({
			resourceQuery: this.resourceQuery,
			query: this.query,
			prev: content.replace(/\r\n?/g, "\n")
		})
	);
};
