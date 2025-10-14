import "./style.css";

const getFile = name =>
	__non_webpack_require__("fs").readFileSync(
		__non_webpack_require__("path").join(__dirname, name),
		"utf-8"
	);

it("should work", () => new Promise(async (resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	try {
		const style = getFile("bundle.css");
		expect(style).toContain("color: red;");
	} catch (e) { }


	await import("./style2.css");

	try {
		const style2 = getFile("style2_css.css");
		expect(style2).toContain("color: red;");
	} catch (e) { }

	NEXT(require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
		try {
			const style = getFile("bundle.css");
			expect(style).toContain("color: blue;");
		} catch (e) { }

		try {
			const style2 = getFile("style2_css.css");
			expect(style2).toContain("color: blue;");
		} catch (e) { }

		done();
	}));
}));

module.hot.accept();
