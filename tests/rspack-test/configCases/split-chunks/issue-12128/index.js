it("should be main", function () {
	require("./a");
	require("./b");

	expect(window["rspackChunk"].length).toBe(1);
});
