import { value as a } from "./a";
import { value as b } from "./b";
import c from "./c";
import state from "./state";

it("should retain async imports until their replacements finish evaluating", async () => {
	expect([a, b, c]).toEqual([1, 1, 1]);
	state.pending = new Promise(resolve => { state.release = resolve; });
	let accepted;
	let duringUpdate;
	module.hot.accept("./a", () => { accepted = a; });
	module.hot.accept("./b");
	module.hot.accept("./c", () => {
		duringUpdate = { started: state.started.slice(), values: [a, b, c] };
		state.release();
	});
	await NEXT_HMR();
	expect(duringUpdate).toEqual({ started: ["a", "b"], values: [1, 1, 2] });
	expect(accepted).toBe(2);
	expect([a, b, c]).toEqual([2, 2, 2]);
});
