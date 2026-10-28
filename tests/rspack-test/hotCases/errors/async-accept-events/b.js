require("./module");
module.hot.accept(
	"./module",
	() => {
		throw new Error("b failed");
	},
	(error, context) => {
		expect(error.message).toBe("b failed");
		expect(context).toEqual({ moduleId: "./b.js", dependencyId: "./module.js" });
		throw new Error("b handler failed");
	}
);
