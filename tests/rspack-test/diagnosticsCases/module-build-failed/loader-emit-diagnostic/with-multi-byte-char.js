/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function() {
	this.experiments.emitDiagnostic({
		message: "Multi-byte character error",
		severity: "error",
		sourceCode: `👯‍♀️👯‍♀️👯‍♀️👯‍♀️`,
		location: {
			line: 1,
			column: 0,
			length: 13,
		},
	});
	// Boundary error
	this.experiments.emitDiagnostic({
		message: "Multi-byte character error",
		severity: "error",
		sourceCode: `"❤️🧡💛💚💙💜"`,
		location: {
			line: 1,
			column: 0,
			length: 13,
		},
	});
	return ""
}
