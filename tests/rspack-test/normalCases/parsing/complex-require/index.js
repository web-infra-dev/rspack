it("should parse template strings in import", async function () {
	var name = "abc".split("");
	var suffix = "Test";
	await Promise.all([
		import(`./abc/${name[0]}${name[1]}${name[2]}Test`),
		import(String.raw`./${name.join("")}/${name.join("")}Test`),
		import(String.raw`./abc/${name.join("")}${suffix}`)
	])
		.then(function (imports) {
			for (var i = 0; i < imports.length; i++) {
				expect(imports[i].default).toEqual("ok");
			}
		})
});

it("should parse .concat strings in import", async function () {
	var name = "abc".split("");
	var suffix = "Test";
	await import("./abc/".concat(name[0]).concat(name[1]).concat(name[2], "Test"))
		.then(function (imported) {
			expect(imported.default).toEqual("ok");
		})
});

require("./cjs")
// require("./amd")
