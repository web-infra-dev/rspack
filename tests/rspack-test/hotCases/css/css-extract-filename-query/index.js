import "./style.css";

function stylesheets() {
	return Array.from(document.getElementsByTagName("link")).filter(function (link) {
		return (link.getAttribute("href") || "").split("?")[0].endsWith("main.css");
	});
}

it("should replace a stylesheet whose filename has a query", async () => {
	if (typeof document === "undefined") return;
	const before = stylesheets();
	expect(before.length).toBe(1);
	await NEXT_HMR();
	const after = stylesheets();
	expect(after.length).toBe(1);
	expect(after[0]).not.toBe(before[0]);
});

module.hot.accept("./style.css");
