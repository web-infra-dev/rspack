import "./foo.css";

it("should handle HMR for exportType style with @import", async () => {
	const styles = window.document.getElementsByTagName("style");
	expect(styles.length).toBeGreaterThan(0);
	const styleElement = styles[styles.length - 1];

	const styleElement2 = styles[styles.length - 2];
	expect(styleElement2.nodeName).toBe("STYLE");
	expect(styleElement2.textContent).toContain("bar-v1");
	expect(styleElement.textContent).toContain(".foo");

	const originalTextContent = styleElement2.textContent;
	await NEXT_HMR();

	const updatedStyles = window.document.getElementsByTagName("style");
	const updatedStyleElement = updatedStyles[updatedStyles.length - 1];
	const updatedStyleElement2 = updatedStyles[updatedStyles.length - 2];

	expect(updatedStyleElement2.textContent).toContain("bar-v2");
	expect(updatedStyleElement.textContent).toContain(".foo");
	expect(updatedStyleElement2.textContent).not.toBe(originalTextContent);
});

module.hot.accept();
