import entry from "./entry";

it("should not throw error when hmr remove circular dependencies", async () => {
	expect(entry).toBe("entry.js");
	await NEXT_HMR();
	expect(entry).toBe("new_entry.js");
});

module.hot.accept("./entry");
