it("should support regex with escape well", () => {
	const info = "..";
	try {
		const hooks = require(info + "/nr-hooks");
		console.log("hooks:", hooks);
	} catch (err) {}
});
