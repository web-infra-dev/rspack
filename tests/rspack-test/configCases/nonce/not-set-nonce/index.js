it("should load script without nonce", function () {
	__webpack_nonce__ = undefined;
	const promise = import(
		"./empty?a" /* webpackChunkName: "chunk-without-nonce" */
	);

	var script = document.head._children.pop();
	require("./chunk-without-nonce.web.js");
	expect(script.getAttribute("nonce")).toBeFalsy();

	return promise;
});
