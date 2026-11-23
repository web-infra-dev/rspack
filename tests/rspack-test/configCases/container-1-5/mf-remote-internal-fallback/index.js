const loadRemote = () => import("app/widget");

it("should build a remote whose first entry is internal", () => {
	expect(typeof loadRemote).toBe("function");
});
