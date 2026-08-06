const path = require("path");
const { rspack } = require("@rspack/core");

const context = path.resolve(
	__dirname,
	"../diagnosticsCases/module-parse-failed/concatenate_parse_module"
);

const defineInvalidExpression = () =>
	new rspack.DefinePlugin({
		DEFINE_VAR: "1 2 3"
	});

const coreOptions = fasterModuleConcatenation => ({
	context,
	entry: "./index.js",
	mode: "development",
	devtool: false,
	experiments: {
		fasterModuleConcatenation
	},
	optimization: {
		concatenateModules: true,
		minimize: false
	},
	plugins: [defineInvalidExpression()]
});

const expectParseError = diagnostics => {
	expect(diagnostics.errors).toHaveLength(1);
	expect(diagnostics.errors[0].message).toContain("JavaScript parse error");
	expect(diagnostics.warnings).toHaveLength(0);
};

const expectNoDiagnostics = diagnostics => {
	expect(diagnostics.errors).toHaveLength(0);
	expect(diagnostics.warnings).toHaveLength(0);
};

/** @type {import('@rspack/test-tools').TErrorCaseConfig[]} */
module.exports = [
	{
		description:
			"should use the legacy concatenated-module parser when the experiment is disabled",
		options: () => coreOptions(false),
		check: expectParseError
	},
	{
		description:
			"should skip the concatenated-module parser when the experiment is enabled",
		options: () => coreOptions(true),
		check: expectNoDiagnostics
	}
];
