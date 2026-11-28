import { base } from "./shared.js";

let value;

it("should settle the bundle only after the last async entry finished", () => {
	expect(value).toBe(43);
});

value = await new Promise((resolve) => {
	setTimeout(() => resolve(base + 2), 50);
});
