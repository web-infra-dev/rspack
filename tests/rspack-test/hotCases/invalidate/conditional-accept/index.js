import "./data.json";
import mod1 from "./module1";
import mod2 from "./module2";
import { value1, value2 } from "./store";

it("should invalidate a self-accepted module", async () => {
	const done = err => (err ? reject(err) : resolve());
	expect(mod1).toBe(1);
	expect(mod2).toBe(1);
	expect(value1).toBe(1);
	expect(value2).toBe(1);
	let step = 0;

	await NEXT_HMR();
	expect(mod1).toBe(1);
	expect(mod2).toBe(1);
	expect(value1).toBe(2);
	expect(value2).toBe(2);
	await NEXT_HMR();
	expect(mod1).toBe(2);
	expect(mod2).toBe(2);
	expect(value1).toBe(2);
	expect(value2).toBe(2);
	await NEXT_HMR();
	expect(mod1).toBe(3);
	expect(mod2).toBe(3);
	expect(value1).toBe(3);
	expect(value2).toBe(3);
});

module.hot.accept(["./module1", "./module2", "./data.json"]);
