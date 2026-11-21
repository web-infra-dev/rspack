import { Template } from "@rspack/core";

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

describe("Template.getModulesArrayBounds", () => {
	it("should return false for empty arrays and negative or non-integer module ids", () => {
		expect(Template.getModulesArrayBounds([])).toBe(false);
		expect(Template.getModulesArrayBounds([{ id: -1 }])).toBe(false);
		expect(Template.getModulesArrayBounds([{ id: 1.5 }])).toBe(false);
		expect(Template.getModulesArrayBounds([{ id: Number.NaN }])).toBe(false);
	});

	it("should return bounds for dense non-negative integer module ids", () => {
		expect(Template.getModulesArrayBounds([{ id: 0 }, { id: 1 }, { id: 2 }])).toEqual([0, 2]);
	});
});
