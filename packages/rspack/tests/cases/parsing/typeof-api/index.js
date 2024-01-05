it("should evaluate api typeof", function () {
	expect(require("./typeof")).toEqual({
		require: "function",
		__webpack_is_included__: "function"
	});
});

it("should not parse filtered stuff", function () {
	if (typeof require !== "function") require("fail");
	if (typeof __webpack_hash__ !== "string") require("fail");
	if (typeof __webpack_public_path__ !== "string") require("fail");
	if (typeof __webpack_modules__ !== "object") require("fail");
	if (typeof __webpack_module__ !== "object") require("fail");
	if (typeof __resourceQuery !== "string") require("fail");
	if (typeof __webpack_chunk_load__ !== "function") require("fail");
	if (typeof __webpack_base_uri__ !== "string") require("fail");
	if (typeof __system_context__ !== "object") require("fail");
	if (typeof __webpack_share_scopes__ !== "object") require("fail");
	if (typeof __webpack_init_sharing__ !== "function") require("fail");
	if (typeof __webpack_nonce__ !== "string") require("fail");
	if (typeof __webpack_chunkname__ !== "string") require("fail");
	// if (typeof __webpack_is_included__ !== "function") require("fail"); // Webpack also can't eval `typeof __webpack_is_included !== "function"`
	if (typeof __webpack_require__ !== "function") require("fail");
});
