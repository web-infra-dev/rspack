it("should exist request, rawRequest, and userRequest", async () => {
	const result = require("./a");
	const { request, userRequest, rawRequest } = result;
	expect(request.endsWith("a.js")).toBe(true);
	expect(userRequest.endsWith("a.js")).toBe(true);
	expect(rawRequest).toBe("./a");
});
