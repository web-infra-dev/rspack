it("should exist request, rawRequest, and userRequest", () => {
	const result = require("./a");
	const { request, userRequest, rawRequest, prev} = result;
	expect(request.endWiths("a.js")).toBe(true);
	expect(userRequest.endWiths("a.js")).toBe(true);
	expect(rawRequest.endWiths("a.js")).toBe(true);
	expect(prev).toEqual('module.exports = "a";\n');
});
