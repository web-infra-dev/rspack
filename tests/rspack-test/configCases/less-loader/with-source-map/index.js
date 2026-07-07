const fs = require("fs");
const path = require("path");

it("basic", () => {
	const css = require("./index.less");
	expect(css).toEqual(nsObj({}));
	const sourceMap = fs.readFileSync(__dirname + "/bundle0.css.map", "utf-8");
	const map = JSON.parse(sourceMap);
	let source = "webpack:///./index.less";
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		source = "rspack:///./index.less";
	}
	expect(map.sources).toContain(source);
	expect(map.file).toEqual("bundle0.css");
	expect(map.sourcesContent).toEqual([
		fs.readFileSync(
			__dirname + "/" + require("!!./index.less?resource"),
			"utf-8"
		)
	]);

	const jsSourceMap = fs.readFileSync(__dirname + "/bundle0.js.map", "utf-8");
	const jsMap = JSON.parse(jsSourceMap);
	expect(jsMap.sources).not.toContain(source);
	expect(jsMap.sources).toContain(source.replace("./index.less", "css ./index.less"));
});
