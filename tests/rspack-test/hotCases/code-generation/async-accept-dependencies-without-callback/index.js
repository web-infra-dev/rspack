import value from "./a";
import * as other from "./b";

it("should await async ESM dependencies without an accept callback", async () => {
	expect([value, other.value]).toEqual([1, 1]);
	import.meta.webpackHot.accept(["./a", "./b"]);
	await NEXT_HMR();
	expect([value, other.value]).toEqual([2, 2]);
});
