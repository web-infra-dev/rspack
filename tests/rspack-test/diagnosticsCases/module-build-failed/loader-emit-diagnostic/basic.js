/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function() {
	this.experiments.emitDiagnostic({
		message: "`React` is not defined",
		severity: "error",
	});
	this.experiments.emitDiagnostic({
		message: "`React` is not defined",
		severity: "warning",
	});
	return ""
}
