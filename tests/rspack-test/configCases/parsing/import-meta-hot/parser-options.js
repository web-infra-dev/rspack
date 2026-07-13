const fs = require("fs");

it("should disable the hot alias independently from the canonical HMR field", () => {
	expect(typeof import.meta.hot).toBe("undefined");
	expect(typeof import.meta.webpackHot).toBe("object");
	if (import.meta.hot) {
		import.meta.hot.accept();
		import.meta.hot.decline();
	}

	const source = fs.readFileSync(__filename, "utf-8");
	const importMeta = ["import", "meta"].join(".");
	expect(source.split(`${importMeta}.hot`)).toHaveLength(5);
	expect(source).not.toContain(`${importMeta}.webpackHot`);
});
