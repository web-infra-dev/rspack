it('should evaluate `typeof import.meta.resolve` to "function"', () => {
	expect(typeof import.meta.resolve).toBe("function");
});

it("should evaluate `import.meta.resolve` as a truthy expression", () => {
	let id;

	if (import.meta.resolve) {
		id = import.meta.resolve("./a.js");
	}

	expect(id).toBe(require.resolve("./a.js"));

	const ternaryId = import.meta.resolve
		? import.meta.resolve("./b.js")
		: null;

	expect(ternaryId).toBe(require.resolve("./b.js"));
});

it("should resolve statically analyzable string requests", () => {
	expect(import.meta.resolve("./a.js")).toBe(require.resolve("./a.js"));
	expect(import.meta.resolve("./" + "b.js")).toBe(require.resolve("./b.js"));
	expect(import.meta.resolve(`./${"a"}.js`)).toBe(require.resolve("./a.js"));
});

it("should resolve dynamic requests with context modules", () => {
	function getFile() {
		return "a";
	}

	expect(import.meta.resolve("./dir/" + getFile() + ".js")).toBe(require.resolve("./dir/" + getFile() + ".js"));
	expect(import.meta.resolve(`./dir/${getFile()}.js`)).toBe(require.resolve(`./dir/${getFile()}.js`));

	const flag = Math.random() > 0.5;
	const aId = require.resolve("./a.js");
	const templateId = require.resolve(`./dir/${getFile()}.js`);
	expect(import.meta.resolve(flag ? "./a.js" : `./dir/${getFile()}.js`)).toBe(flag ? aId : templateId);
});
