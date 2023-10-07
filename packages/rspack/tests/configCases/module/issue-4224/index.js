it("should generate asset/resource", () => {
	expect(require("./index.scss").endsWith(".scss")).toBeTruthy();
});
