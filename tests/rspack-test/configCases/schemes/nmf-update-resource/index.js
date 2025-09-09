import a from "data:text/javascript,export default 42;"

it("should build with data uri and after resolve hook", () => {
	expect(a).toBe(42);
});
