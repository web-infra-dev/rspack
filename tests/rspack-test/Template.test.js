const { Template } = require("@rspack/core");

describe("Template.getFunctionContent", () => {
	it("should strip named function wrappers", () => {
		const content = Template.getFunctionContent(function moduleFederationDefaultRuntime_default() {
			return "ok";
		});

		expect(content).toContain('return "ok";');
		expect(content).not.toContain("function moduleFederationDefaultRuntime_default");
	});

	it("should strip anonymous function wrappers", () => {
		const content = Template.getFunctionContent(function () {
			return "ok";
		});

		expect(content).toContain('return "ok";');
		expect(content).not.toContain("function ()");
	});
});
