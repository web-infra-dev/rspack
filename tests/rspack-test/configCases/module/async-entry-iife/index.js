const result = new Promise(resolve => setTimeout(() => resolve(42), 10));

it("should run an async entry inside a synchronous IIFE", async () => {
	expect(await result).toBe(42);
});

await result;

export {};
