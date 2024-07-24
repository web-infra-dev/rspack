/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function () {
	var err = new Error("Message");
	err.stack = "Stack";
	//@ts-expect-error hideStack is not a property on normal errors
	err.hideStack = true;
	throw err;
};
