/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function () {
	var err = new Error("throw message");
	err.stack = "throw stack";
	throw err;
};
