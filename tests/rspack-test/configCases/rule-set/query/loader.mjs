/** @type {import("@rspack/core").LoaderDefinition<{ get(): string }>} */
export default function (source) {
	var query = this.query;
	if (typeof query === "object" && typeof query.get === "function") {
		query = query.get();
	}
	return source + "\nmodule.exports.push(" + JSON.stringify(query) + ");";
};
