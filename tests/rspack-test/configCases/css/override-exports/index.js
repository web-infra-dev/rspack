import * as classes from "./index.module.css";

it("should have correct classes", function () {
	expect(classes).toEqual(nsObj({
		base: "index_module_css-base",
		first: "index_module_css-first index_module_css-base",
		second: "index_module_css-second index_module_css-base",
		container: "index_module_css-container",
	}))
});
