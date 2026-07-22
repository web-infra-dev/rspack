export function load(value) {


	return value ? import("./b") : import("./a");
}

it("should load the expected async module", async () => {
	expect((await load(true)).default).toBe("b");
	expect((await load(false)).default).toBe("a");
});
