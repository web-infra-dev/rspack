it("should evaluate api typeof", function () {
	expect(require("./typeof")).toEqual({
		require: "function",
		__webpack_hash__: "string",
		__webpack_public_path__: "string",
		__webpack_modules__: "object",
		__webpack_module__: "object",
		__resourceQuery: "string",
		__webpack_chunk_load__: "function",
		__webpack_base_uri__: "string",
		__system_context__: "object",
		__webpack_share_scopes__: "object",
		__webpack_init_sharing__: "function",
		__webpack_nonce__: "string",
		__webpack_chunkname__: "string",
		__webpack_is_included__: "function",
		__webpack_require__: "function"
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
	if (typeof __webpack_is_included__ !== "function") require("fail");
	if (typeof __webpack_require__ !== "function") require("fail");
});
