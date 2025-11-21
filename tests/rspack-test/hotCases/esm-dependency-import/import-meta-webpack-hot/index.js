import { val } from "./module";

it("should accept changes", async () => {
	expect(val).toBe(1);
	await NEXT_HMR();
});
