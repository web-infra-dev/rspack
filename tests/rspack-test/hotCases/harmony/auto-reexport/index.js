import { value } from "./reexport";

it("should auto-reexport an ES6 imported value on accept", function (done) {
	expect(value).toBe(1);
	NEXT(
		require("@rspack/test-tools/helper/legacy/update")(done, true, () => {
			expect(value).toBe(2);
			done();
		})
	);
});
