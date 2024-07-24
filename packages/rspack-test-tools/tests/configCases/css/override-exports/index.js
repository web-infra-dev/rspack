import * as classes from "./index.module.css";

it("should have correct classes", function () {
	expect(classes).toEqual(nsObj({
		base: "-_index_module_css-base",
		first: "-_index_module_css-first -_index_module_css-base",
		second: "-_index_module_css-second -_index_module_css-base",
		container: "-_index_module_css-container",
	}))
});
