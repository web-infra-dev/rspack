/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function () {
	var err = new Error("throw message hide");
	err.stack = "throw stack hide";
	err.hideStack = true;
	throw err;
};
