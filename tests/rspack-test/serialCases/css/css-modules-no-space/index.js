const prod = process.env.NODE_ENV === "production";

it("should allow to create css modules", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	require("./use-style_js.bundle0.js");
	import("./use-style.js").then(({ default: x }) => {
		try {
			expect(x).toEqual({
				class: undefined
			});

			const fs = require("fs");
			const path = require("path");
			const cssOutputFilename = "use-style_js.bundle0.css";

			const cssContent = fs.readFileSync(
				path.join(__dirname, cssOutputFilename),
				"utf-8"
			);
			expect(cssContent).toContain("no-space");
			expect(cssContent).toContain("color: red");
		} catch (e) {
			return done(e);
		}
		done();
	}, done);
}));
