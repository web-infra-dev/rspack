import a from "./a";
import b from "./b";

it("should await shared async dependencies independently in each accept callback", async () => {
	expect([a, b]).toEqual([2, 3]);
	const accepted = {};
	module.hot.accept("./a", () => { accepted.a = a; });
	module.hot.accept("./b", () => { accepted.b = b; });
	await NEXT_HMR();
	expect(accepted).toEqual({ a: 3, b: 4 });
	expect([a, b]).toEqual([3, 4]);
});
