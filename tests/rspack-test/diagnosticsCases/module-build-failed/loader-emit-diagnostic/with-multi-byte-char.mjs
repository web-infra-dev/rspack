/** @type {import("@rspack/core").LoaderDefinition} */
export default function() {
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
