import * as a from "./a";

const nsObj = m => {
	Object.defineProperty(m, Symbol.toStringTag, { value: "Module" });
	return m;
};
debugger
it("should have to correct exports", () => {
	debugger
	expect(a).toStrictEqual(nsObj({
		[`x${WATCH_STEP}`]: WATCH_STEP
	}));
})
