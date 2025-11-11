
it("should not parse filtered stuff", function () {
	if (typeof __rspack_hash__ !== "string") require("fail");
	if (typeof __rspack_public_path__ !== "string") require("fail");
	if (typeof __rspack_modules__ !== "object") require("fail");
	if (typeof __rspack_module__ !== "object") require("fail");
	if (typeof __rspack_chunk_load__ !== "function") require("fail");
	if (typeof __rspack_base_uri__ !== "string") require("fail");
	if (typeof __rspack_share_scopes__ !== "object") require("fail");
	if (typeof __rspack_init_sharing__ !== "function") require("fail");
	if (typeof __rspack_nonce__ !== "string") require("fail");
	if (typeof __rspack_chunkname__ !== "string") require("fail");
	if (typeof __rspack_get_script_filename__ !== "function") require("fail");
	if (typeof __rspack_require__ !== "function") require("fail");
	if (typeof __rspack_version__ !== "string") require("fail");
	if (typeof __rspack_unique_id__ !== "string") require("fail");
});
