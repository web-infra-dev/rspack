/** @type {import("@rspack/core").LoaderDefinition} */
export default function() {
	this.experiments.emitDiagnostic({
		message: "`React` is not defined",
		severity: "error",
		sourceCode: `<div></div>`,
		help: "try to import `React`",
		location: {
			line: 1,
			column: 1,
			length: 3,
		}
	});
	return ""
}

