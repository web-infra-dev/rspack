import "./main";

it("css recovery", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	NEXT(
		require("../../update")(
			err => {
				expect(String(err)).toContain("Module build failed");
				NEXT(require("../../update")(done, true, () => done()));
			},
		)
	);
}));
