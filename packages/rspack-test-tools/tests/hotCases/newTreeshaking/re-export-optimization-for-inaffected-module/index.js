import {value} from "./module";

let v = value;
module.hot.accept('./module', () => {
	v = value
});

it("should auto-reexport an ES6 imported value on accept with newTreeshaking", async function (done) {
	expect(v).toBe("foo");
	NEXT(
		require("../../update")(done, true, () => {
			expect(v).toBe("foo");
			done();
		})
	);
});
