let ready = false;

it("should await an async entry before a synchronous last entry", () => {
	expect(ready).toBe(true);
});

await new Promise(resolve => setTimeout(resolve, 50));
ready = true;

export {};
