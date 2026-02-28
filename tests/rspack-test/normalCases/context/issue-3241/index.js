it("should support regex with escape well", () => {
	let info = "..";
	try {
		const hooks = require(info + "/nr-hooks");
		console.log("hooks:", hooks);
	} catch (err) {}
});
