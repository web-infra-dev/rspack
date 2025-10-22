import { load } from "./parent-file";

it("should bubble update from a nested dependency", async () => {
	let value = await load();
	expect(value).toBe(1);
	await NEXT_HMR();
	value = await load();
	expect(value).toBe(2);
});

module.hot.accept("./parent-file");