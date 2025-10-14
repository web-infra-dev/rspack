import "./main";

it("css recovery", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	NEXT(
		require("@rspack/test-tools/helper/legacy/update")(
			err => {
				expect(String(err)).toContain("Module build failed");
				NEXT(require("@rspack/test-tools/helper/legacy/update")(done, true, () => done()));
			},
		)
	);
}));
