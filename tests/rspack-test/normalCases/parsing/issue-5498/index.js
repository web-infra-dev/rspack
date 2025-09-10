it("require alias should works", () => {
	const r = typeof require === "function" && require;
	expect(r("./foo")).toBe("foo");

	if (typeof require === "function") {
		// call expression is ok
		expect(require("./foo")).toBe("foo");
	}

	if (typeof require === "function") {
		// identifier case 2
		const r = require;
		expect(r("./foo")).toBe("foo");
	}
});
