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

it("should have files", async () => {
	const fs = require("fs");
	const path = require("path");
	const files = await fs.promises.readdir(__dirname);
	expect(files).toContain("shared-shared-shared1_js.js");
	expect(files).toContain("shared-shared-shared2_js.js");
	expect(files).toContain("splitted-chunks");
	const innerFiles = await fs.promises.readdir(path.resolve(__dirname, "splitted-chunks"));
	expect(innerFiles).toContain("common-common1_js.js");
	expect(innerFiles).toContain("common-common2_js.js");
	expect(innerFiles).toContain("common-common3_js.js");
});
