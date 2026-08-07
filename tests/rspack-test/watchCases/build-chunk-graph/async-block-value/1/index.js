export function load(value) {


	return value ? import("./a") : import("./b");
}

it("should load the expected async module", async () => {
	expect((await load(true)).default).toBe("a");
	expect((await load(false)).default).toBe("b");
});
