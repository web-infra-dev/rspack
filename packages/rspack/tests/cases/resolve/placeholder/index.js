it("should resolve [] syntax in path", function () {
	var a = require("./[id].js");
	expect(typeof a).toEqual("object");
});
