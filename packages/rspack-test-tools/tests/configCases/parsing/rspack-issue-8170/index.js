it("should define __dirname", function() {
	expect(require("./dir1/dir2/file").dirname).toMatch(/^dir1[\\\/]dir2$/);
});
