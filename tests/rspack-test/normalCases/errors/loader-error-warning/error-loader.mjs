/** @type {import("@rspack/core").LoaderDefinition<string>} */
export default function (source) {
	//@ts-expect-error errors must be Errors, string is not recommended and should lead to type error
	this.emitError(this.query.slice(1));
	return source;
};
