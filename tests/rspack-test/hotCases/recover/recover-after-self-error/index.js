import getValue, { getError, id } from "./a";

const moduleId = id;

it("should abort when module is not accepted", async () => {
	expect(getValue()).toBe(1);
	expect(getError()).toBe(false);
	await NEXT_HMR();
	expect(getValue()).toBe(2);
	expect(getError()).toBe(true);
	await NEXT_HMR();
	expect(getValue()).toBe(2);
	expect(getError()).toBe(true);
	await NEXT_HMR();
	expect(getValue()).toBe(4);
	expect(getError()).toBe(false);
});

module.hot.accept("./a");
