import "./style.css";

it("should compile and load style on demand", () => {
	const link = document.getElementsByTagName("link")[0];
	expect(getLinkSheet(link)).toContain("background: red;");
});
