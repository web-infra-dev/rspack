import "./index.css";

it("keeps the media attribute of the replaced stylesheet", async () => {
	document.head.children[0].setAttribute("media", "(prefers-color-scheme: dark)");
	await NEXT_HMR();
	const [link] = document.head.getElementsByTagName("link");
	expect(link.href).toContain("bundle.css?hmr");
	expect(link.getAttribute("media")).toBe("(prefers-color-scheme: dark)");
});

module.hot.accept("./index.css");
