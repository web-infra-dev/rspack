require("./module");
module.hot.accept("./module", () => {
	throw new Error("a failed");
});
