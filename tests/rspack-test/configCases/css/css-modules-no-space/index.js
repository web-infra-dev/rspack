
const prod = process.env.NODE_ENV === "production";

it("should allow to create css modules", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	__non_webpack_require__("./use-style_js.bundle0.js");
	import("./use-style.js").then(({ default: x }) => {
		try {

			const fs = __non_webpack_require__("fs");
			const path = __non_webpack_require__("path");

			expect(x).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, 'x.txt'));

			const cssOutputFilename = "use-style_js.bundle0.css";

			const cssContent = fs.readFileSync(
				path.join(__dirname, cssOutputFilename),
				"utf-8"
			);
			expect(cssContent).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, 'cssContent.txt'));
		} catch (e) {
			return done(e);
		}
		done();
	}, done);
}));
