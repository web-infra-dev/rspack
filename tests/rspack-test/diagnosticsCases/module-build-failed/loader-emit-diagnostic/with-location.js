/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function() {
	this.experiments.emitDiagnostic({
		message: "`React` is not defined",
		severity: "error",
		sourceCode: `<div></div>`,
		location: {
			line: 1,
			column: 1,
			length: 3,
		}
	});
	return ""
}

