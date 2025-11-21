it("should load script with nonce 'nonce1234'", function () {
	__webpack_nonce__ = "nonce1234";
	const promise = import(
		"./empty?a" /* webpackChunkName: "chunk-with-nonce" */
	);

	var script = document.head._children.pop();
	__non_webpack_require__("./chunk-with-nonce.web.js");
	expect(script.getAttribute("nonce")).toBe("nonce1234");

	return promise;
});
