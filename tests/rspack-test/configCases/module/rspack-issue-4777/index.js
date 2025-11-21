it("should loader the correctly query", () => {
	expect(require("./sub/a")).toBe("?query=a");
	expect(require('./sub/b')).toBe('?query=alias')
	expect(require('./sub/c')).toBe('?query=c')
	expect(require("./sub/d")).toBe("?query=d");
	expect(require("./sub/e")).toBe("?query=options-e");
	expect(require("./sub/f")).toStrictEqual({
		query: "options-object-f"
	});
	expect(require("./sub/g")).toStrictEqual({
		query: "options-object-g"
	});
	expect(require("./sub/h")).toStrictEqual({
		query: "options-object-h"
	});
});
