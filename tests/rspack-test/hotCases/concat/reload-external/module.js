import value1 from "./a";
import value2 from "./b";

it("should allow to hot replace modules in a ConcatenatedModule", async () => {
	expect(value1).toBe(1);
	expect(value2).toBe(10);
	await NEXT_HMR();
	expect(value1).toBe(2);
	await NEXT_HMR();
	expect(value2).toBe(20);
});

module.hot.accept(["./a", "./b"]);
