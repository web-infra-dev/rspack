import "./shared1";
import "./common1";

it("should be able to load the split chunk on demand (shared)", () => {
	return import(/* webpackChunkName: "theName" */ "./shared2");
});

it("should be able to load the split chunk on demand (common)", () => {
	return Promise.all([
		import(/* webpackChunkName: "otherName1" */ "./common2"),
		import(/* webpackChunkName: "otherName2" */ "./common3")
	]);
});

it("should have files", () => {
	const files = require("fs").readdirSync(__dirname);
	expect(files).toContain("a.js");
	expect(files).toContain("b.js");
	expect(files).toContain("common-common1_js.js");
	expect(files).toContain("otherName1.js");
	expect(files).toContain("otherName2.js");
	expect(files).toContain("shared-shared-shared1_js.js");
	expect(files).toContain("shared-theName.js");
});
