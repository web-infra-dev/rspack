import "./index.css";

it("css recovery", done => {
	NEXT(
		require("../../update")(err => {
			expect(String(err)).toContain("Module build failed");
			NEXT(require("../../update")(done, true, () => done()));
		})
	);
});
