it("should exist request, rawRequest, and userRequest", async () => {
	const result = require("./a");
	const { request, userRequest, rawRequest } = result;
	expect(request.endWiths("a.js")).toBe(true);
	expect(userRequest.endWiths("a.js")).toBe(true);
	expect(rawRequest.endWiths("a.js")).toBe(true);
});
