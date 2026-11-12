/** @type {import("@rspack/core").LoaderDefinition} */
export default function (source) {
	const empty = null;
	const emptyError = new Error();
	this.emitWarning(empty);
	this.emitWarning(emptyError);
	this.emitError(empty);
	this.emitError(emptyError);
	throw "a string error";
	return source;
};
