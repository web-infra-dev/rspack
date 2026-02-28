beforeEach(() => {
	global.it = it;
});
afterEach(() => {
	delete global.it;
});

it("should be able to load the other entry on demand", () => import("./a"));
