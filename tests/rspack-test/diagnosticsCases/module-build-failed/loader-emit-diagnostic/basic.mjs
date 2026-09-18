/** @type {import("@rspack/core").LoaderDefinition} */
export default function() {
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
