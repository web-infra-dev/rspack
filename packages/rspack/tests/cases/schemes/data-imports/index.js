import a from 'data:text/javascript,export default "a";';

it("data imports", () => {
	expect(a).toBe("a");
});
