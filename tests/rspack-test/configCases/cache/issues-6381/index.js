const a = "a";
import(`./a/${a}.js`);
it("should work when snapshot strategies only set { timestamp: true }", () => {
	expect(require("./a/a").default).toBe(1);
});
