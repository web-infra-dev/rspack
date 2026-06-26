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

	expect(typeof import.meta.rspackVersion).toBe("string");
	expect(import.meta.rspackVersion.length > 0).toBe(true);

	expect(typeof import.meta.rspackHash).toBe("string");
	expect(import.meta.rspackHash.length > 0).toBe(true);

	function callRspackInitSharing() {
		return import.meta.rspackInitSharing("default");
	}
	expect(typeof callRspackInitSharing).toBe("function");

	const {
		rspackPublicPath,
		rspackInitSharing,
		rspackVersion,
		rspackHash
	} = import.meta;
	expect(rspackPublicPath).toBe("/a");
	expect(rspackInitSharing).toBe(__webpack_require__.I);
	expect(typeof rspackVersion).toBe("string");
	expect(rspackVersion.length > 0).toBe(true);
	expect(typeof rspackHash).toBe("string");
	expect(rspackHash.length > 0).toBe(true);
});
