const { formatDiagnostic } = require("./binding")

const diagnostic = {
  name: "ModuleError",
  message: "failed to link",
  severity: "error",
	sourceCode: `abc;
def;
ghi;`,
help: "Try to fix it",
	location: {
		text: "abc",
		line: 1,
		column: 1,
		length: 1,
	},
  module_identifier: "test",
  file: "test",
}

formatDiagnostic(diagnostic)
