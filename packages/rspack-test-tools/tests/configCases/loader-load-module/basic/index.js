import a from "./loader.js!./a";

it("should loaderContext.loadModule works", () => {
	expect(a).toBe(1);
});
