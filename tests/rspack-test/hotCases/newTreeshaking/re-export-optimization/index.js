import value from "./entry";

let v = value;
module.hot.accept('./entry', () => {
	v = value
});

it("should auto-reexport an ES6 imported value on accept with newTreeshaking", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(v).toBe("foo");
	NEXT(
		require("../../update")(done, true, () => {
			expect(v).toBe("foobar");
			done();
		})
	);
}));
