import entry from "./entry";

it("should not throw error when hmr remove circular dependencies", done => {
	expect(entry).toBe("entry.js");
	module.hot.accept("./entry", () => {
		expect(entry).toBe("new_entry.js");
		done();
	});
	NEXT(require("../../update")(done));
});
