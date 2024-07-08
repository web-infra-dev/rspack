const fs = require("fs");

try {
	require("./index.module.css");
} catch (e) {
	// ignore
}

it("css modules classname with default output hash options should works", () => {
	const code = fs.readFileSync(__filename, "utf-8");
	const name = /"__LOCAL_CLASS_NAME__": [`'"](.*)[`'"]/.exec(code)[1];
	expect(name).toEqual("index-module____LOCAL_CLASS_NAME__--b4232134");
});
