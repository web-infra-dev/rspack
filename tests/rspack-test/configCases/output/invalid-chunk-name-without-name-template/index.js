const load = () => import(/* webpackChunkName: "chunk?query" */ "./async");

it("should not warn without a static [name] template", () => {
	expect(load).toBeInstanceOf(Function);
});
