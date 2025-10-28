import "./modules/a";

it("should load", async () => {
	await Promise.all([import("./modules/b"), import("./modules/c")]);
});
