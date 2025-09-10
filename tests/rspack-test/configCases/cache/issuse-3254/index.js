const a = "a";
import(`./a/${a}.js`);
it("should work with cache", () => {
	expect(require("./a/a").default).toBe(1);
});
