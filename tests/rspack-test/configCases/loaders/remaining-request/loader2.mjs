/** @type {import("@rspack/core").LoaderDefinition<{ f(): any }>} */
export default function (source) {
	if (typeof this.query === "string")
		throw new Error("query must be an object");
	return "module.exports = " + JSON.stringify(this.query.f());
};
