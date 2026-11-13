it("shares writable runtime globals between entry and async chunks", async () => {
	const read = () => [__webpack_public_path__, __webpack_nonce__];
	expect(read()).toEqual(["/initial/", undefined]);
	__webpack_public_path__ = "/entry/";
	__webpack_nonce__ = "entry-nonce";

	const asyncModule = await import("./async.js");
	expect(asyncModule.read()).toEqual(["/entry/", "entry-nonce"]);
	asyncModule.write("/async/", "async-nonce");
	expect(read()).toEqual(["/async/", "async-nonce"]);

	__webpack_public_path__ = "/updated/";
	__webpack_nonce__ = "updated-nonce";
	expect(asyncModule.read()).toEqual(["/updated/", "updated-nonce"]);
});
