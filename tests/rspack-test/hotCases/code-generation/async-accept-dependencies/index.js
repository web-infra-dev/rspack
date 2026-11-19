import value, { label } from "./a";
import * as other from "./b";
import sync from "./sync";

it("should await async ESM dependencies before calling accept", async () => {
	expect([value, label, other.value, sync]).toEqual([1, "a", 1, 1]);
	let accepted;
	import.meta.webpackHot.accept(["./a", "./b", "./sync"], () => {
		accepted = [value, label, other.value, sync];
	});
	await NEXT_HMR();
	expect(accepted).toEqual([2, "b", 2, 2]);
	expect([value, label, other.value, sync]).toEqual([2, "b", 2, 2]);
});
