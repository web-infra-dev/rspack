const a = "a";
import(`./a/${a}.js`);
it("should work when snapshot.module or snapshot.resolve only set { timestamp: true }", () => {
	expect(require("./a/a").default).toBe(1);
});
