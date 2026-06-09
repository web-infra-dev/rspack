it("should expose the full security options in federation init options", () => {
	const security = __webpack_require__.federation.initOptions.security;

	expect(security).toEqual({
		allowedRemoteOrigins: ["localhost", "https://cdn.example.com"]
	});
});
