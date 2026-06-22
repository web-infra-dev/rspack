import "./style.css";

const getFile = name =>
	require("fs").readFileSync(
		require("path").join(__dirname, name),
		"utf-8"
	);

it("should work", async () => {
	try {
		const style = getFile("bundle.css");
		expect(style).toContain("color: red;");
	} catch (e) { }


	await import("./style2.css");

	try {
		const style2 = getFile("style2_css.css");
		expect(style2).toContain("color: red;");
	} catch (e) { }

	await NEXT_HMR();

	try {
		const style = getFile("bundle.css");
		expect(style).toContain("color: blue;");
	} catch (e) { }

	try {
		const style2 = getFile("style2_css.css");
		expect(style2).toContain("color: blue;");
	} catch (e) { }
});

module.hot.accept();
