import "./style.css";

it("should compile with warning", async () => {
	// jsdom can not parse @namespace correctly
	const link = document.getElementsByTagName("link")[0];
	expect(getLinkSheet(link)).toContain("background: red;");
});
