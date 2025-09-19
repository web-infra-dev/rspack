const prod = process.env.NODE_ENV === "production";

it("should allow to create css modules", done => {
	__non_webpack_require__("./use-style_js.bundle0.js");
	import("./use-style.js").then(({ default: x }) => {
		try {
			expect(x).toMatchFileSnapshot(`${__SNAPSHOT__}/x.txt`);

			const fs = __non_webpack_require__("fs");
			const path = __non_webpack_require__("path");
			const cssOutputFilename = "use-style_js.bundle0.css";

			const cssContent = fs.readFileSync(
				path.join(__dirname, cssOutputFilename),
				"utf-8"
			);
			expect(cssContent).toMatchFileSnapshot(`${__SNAPSHOT__}/cssContent.txt`);
		} catch (e) {
			return done(e);
		}
		done();
	}, done);
});
