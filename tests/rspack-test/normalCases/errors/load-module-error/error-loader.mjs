/** @type {import("@rspack/core").LoaderDefinition} */
export default function (source) {
	const callback = this.async();
	callback(new Error("err: abc"));
}
