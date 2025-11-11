
it("should parse rspack style api", function () {
	expect(__rspack_hash__).toEqual(__webpack_hash__);
	expect(__rspack_layer__).toEqual(__webpack_layer__);
	expect(__rspack_public_path__).toEqual(__webpack_public_path__);
	expect(__rspack_modules__).toEqual(__webpack_modules__);
	expect(__rspack_module__).toEqual(__webpack_module__);
	expect(__rspack_chunk_load__).toEqual(__webpack_chunk_load__);
	expect(__rspack_base_uri__).toEqual(__webpack_base_uri__);
	expect(__rspack_share_scopes__).toEqual(__webpack_share_scopes__);
	expect(__rspack_init_sharing__).toEqual(__webpack_init_sharing__);
	expect(__rspack_nonce__).toEqual(__webpack_nonce__);
	expect(__rspack_chunkname__).toEqual(__webpack_chunkname__);
	expect(__rspack_runtime_id__).toEqual(__webpack_runtime_id__);
	expect(__rspack_get_script_filename__).toEqual(__webpack_get_script_filename__);
	expect(__rspack_require__).toEqual(__webpack_require__);
	expect(__rspack_module__.id).toEqual(__webpack_module__.id);
});
