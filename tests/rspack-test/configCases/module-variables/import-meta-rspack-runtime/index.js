it("should expose rspack runtime module variables on import.meta", function () {
	expect(typeof import.meta.rspackPublicPath).toBe("string");
	expect(import.meta.rspackPublicPath).toBe("/");
	import.meta.rspackPublicPath = "/a";
	expect(import.meta.rspackPublicPath).toBe("/a");
	expect(__webpack_require__.p).toBe("/a");

	expect(typeof import.meta.rspackBaseUri).toBe("string");
	expect(typeof import.meta.rspackShareScopes).toBe("object");
	const shareScope = { fromImportMeta: true };
	import.meta.rspackShareScopes = {};
	import.meta.rspackShareScopes.__importMetaRuntimeAliasTest = shareScope;
	expect(__webpack_require__.S.__importMetaRuntimeAliasTest).toBe(shareScope);
	expect(typeof import.meta.rspackInitSharing).toBe("function");
	expect(typeof import.meta.rspackNonce).toBe("string");
	expect(typeof import.meta.rspackUniqueId).toBe("string");
	expect(import.meta.rspackUniqueId).toBe(__rspack_unique_id__);

	expect(typeof import.meta.rspackVersion).toBe("string");
	const rspackVersionBeforeAssign = import.meta.rspackVersion;
	expect(rspackVersionBeforeAssign.length > 0).toBe(true);
	import.meta.rspackVersion = "overwritten";
	expect(import.meta.rspackVersion).toBe(rspackVersionBeforeAssign);
	const rspackVersionCompoundAssign =
		(import.meta.rspackVersion += "-suffix");
	expect(rspackVersionCompoundAssign).toBe(
		`${rspackVersionBeforeAssign}-suffix`
	);
	expect(import.meta.rspackVersion).toBe(rspackVersionBeforeAssign);

	expect(typeof import.meta.rspackHash).toBe("string");
	const rspackHashBeforeAssign = import.meta.rspackHash;
	expect(rspackHashBeforeAssign.length > 0).toBe(true);
	import.meta.rspackHash = "overwritten";
	expect(import.meta.rspackHash).toBe(rspackHashBeforeAssign);
	let rspackHashLogicalAssignRhsEvaluated = false;
	const rspackHashLogicalAssign =
		(import.meta.rspackHash ||= (rspackHashLogicalAssignRhsEvaluated = true));
	expect(rspackHashLogicalAssignRhsEvaluated).toBe(false);
	expect(rspackHashLogicalAssign).toBe(rspackHashBeforeAssign);
	expect(import.meta.rspackHash).toBe(rspackHashBeforeAssign);

	function callRspackInitSharing() {
		return import.meta.rspackInitSharing("default");
	}
	expect(typeof callRspackInitSharing).toBe("function");

	const {
		rspackPublicPath,
		rspackInitSharing,
		rspackUniqueId,
		rspackVersion,
		rspackHash
	} = import.meta;
	expect(rspackPublicPath).toBe("/a");
	expect(rspackInitSharing).toBe(__webpack_require__.I);
	expect(rspackUniqueId).toBe(__rspack_unique_id__);
	expect(typeof rspackVersion).toBe("string");
	expect(rspackVersion.length > 0).toBe(true);
	expect(typeof rspackHash).toBe("string");
	expect(rspackHash.length > 0).toBe(true);
});
