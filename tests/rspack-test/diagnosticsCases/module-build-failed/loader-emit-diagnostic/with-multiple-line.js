/** @type {import("@rspack/core").LoaderDefinition} */
module.exports = function() {
	this.experiments.emitDiagnostic({
		message: "Multiple line error",
		severity: "error",
		sourceCode: `~~~~~
~~~~~`,
		location: {
			line: 1,
			column: 0,
			length: 10,
		},
	});
	this.experiments.emitDiagnostic({
		message: "Multiple line error",
		severity: "error",
		sourceCode: `~~~~~
~~~~~`,
		location: {
			text: "unexpected '~'",
			line: 1,
			column: 0,
			length: 10,
		},
	});
	this.experiments.emitDiagnostic({
		message: "Multiple line snippet",
		severity: "error",
		sourceCode: `~~~~~
~~~~~
~~~~~
~~~~~
~~~~~`,
		location: {
			text: "unexpected '~'",
			line: 3,
			column: 4,
			length: 0,
		},
	});
	// Length overflow
	this.experiments.emitDiagnostic({
		message: "Multiple line snippet",
		severity: "error",
		sourceCode: `~~~~~
~~~~~
~~~~~
~~~~~
~~~~~`,
		location: {
			text: "unexpected '~'",
			line: 3,
			column: 4,
			length: 100,
		},
	});
	return ""
}
