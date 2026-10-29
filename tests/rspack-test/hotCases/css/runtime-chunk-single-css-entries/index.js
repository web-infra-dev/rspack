const stylesheets = name =>
	Array.from(document.getElementsByTagName("link")).filter(link =>
		(link.getAttribute("href") || "").split("?")[0].endsWith(name)
	);

it("only updates stylesheets of entries loaded on the page", async () => {
	const link = document.createElement("link");
	link.rel = "stylesheet";
	link.href = __webpack_require__.p + "a.css";
	await new Promise((resolve, reject) => {
		link.onload = resolve;
		link.onerror = reject;
		document.head.appendChild(link);
	});

	await NEXT_HMR();

	expect(stylesheets("a.css").map(l => l.getAttribute("href"))).toEqual([
		expect.stringContaining("a.css?hmr=")
	]);
	expect(stylesheets("b.css")).toHaveLength(0);
	expect(stylesheets("c.css")).toHaveLength(0);
});

module.hot.accept();
