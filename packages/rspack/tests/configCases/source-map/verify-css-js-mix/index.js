const checkMap = require("../checkSourceMap").default;
const fs = require("fs");
const path = require("path");

try {
	require("./a.js");
} catch (e) {
	// ignore
}

it("verify importing css js source map", async () => {
	const source = fs.readFileSync(__filename + ".map", "utf-8");
	const map = JSON.parse(source);
	expect(map.sources).toEqual(["../checkSourceMap.js", "./a.js", "./index.js"]);
	expect(map.file).toEqual("main.js");
	const out = fs.readFileSync(__filename, "utf-8");
	expect(
		await checkMap(out, source, {
			// *${id}* as the search key to aviod conflict with `Object.defineProperty(exports, ${id}, ...)`
			["*a0*"]: "a.js",
			["*a1*"]: "a.js"
		})
	).toBe(true);
});

it("verify css source map", async () => {
	const cssSource = fs.readFileSync(
		path.resolve(__dirname, "main.css.map"),
		"utf-8"
	);
	const cssMap = JSON.parse(cssSource);
	expect(cssMap.sources).toEqual(["./a.css"]);
	expect(cssMap.file).toEqual("main.css");
	const cssOut = fs.readFileSync(path.resolve(__dirname, "main.css"), "utf-8");
	expect(
		await checkMap(cssOut, cssSource, {
			a0: "a.css",
			a1: "a.css",
			a2: "a.css"
		})
	).toBe(true);
});
