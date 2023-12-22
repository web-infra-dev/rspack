it("should have named natural ids", function () {
	for (var i = 1; i <= 5; i++) {
		const moduleId = require("./files/file" + i + ".js");
		// console.log(i, moduleId)
		expect(moduleId).toMatch(String(i + 1));
	}
});
