/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function () {
	var err = new Error("emit message hide");
	err.stack = "emit stack hide";
	err.hideStack = true;
	this.emitError(err);
	return '';
};
