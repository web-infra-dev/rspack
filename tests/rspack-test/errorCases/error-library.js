/** @type {import('@rspack/test-tools').TErrorCaseConfig} */
module.exports = {
	description: "should throw error for invalid library name",
	options() {
		return {
			entry: "./file",
			output: {
				libraryTarget: "var"
			}
		};
	},
	async check(diagnostics) {
		expect(diagnostics).toMatchInlineSnapshot(`
		Object {
		  "errors": Array [
		    Object {
		      "code": "GenericFailure",
		      "message": "  × caused by plugins in Compilation.hooks.additionalChunkRuntimeRequirements  ╰─▶   × Library name must be a string or string array. Common configuration options that specific library names are 'output.library[.name]', 'entry.xyz.library[.name]', 'ModuleFederationPlugin.name' and 'ModuleFederationPlugin.library[.name]'.      ",
		      "stack": "Error:   × caused by plugins in Compilation.hooks.additionalChunkRuntimeRequirements  ╰─▶   × Library name must be a string or string array. Common configuration options that specific library names are 'output.library[.name]', 'entry.xyz.library[.name]', 'ModuleFederationPlugin.name' and 'ModuleFederationPlugin.library[.name]'.      ",
		    },
		  ],
		  "warnings": Array [],
		}
	`);
	}
};
