import value from "./entry";

let v = value;
module.hot.accept('./entry', () => {
	v = value
});

it("should auto-reexport an ES6 imported value on accept with newTreeshaking", async () => {
	expect(v).toBe("foo");
	await NEXT_HMR();
	expect(v).toBe("foobar");
});
