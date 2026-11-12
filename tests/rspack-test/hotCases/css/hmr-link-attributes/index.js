import "./index.css";

it("preserves stylesheet attributes and disabled state across hot updates", async () => {
	const attributes = {
		media: "(prefers-color-scheme: dark)",
		title: "theme",
		"data-theme": "dark"
	};
	const discarded = ["integrity", "onload", "onerror"];
	const original = document.querySelector("link");
	for (const [name, value] of Object.entries(attributes)) {
		original.setAttribute(name, value);
	}
	for (const name of discarded) original.setAttribute(name, "stale");
	original.nonce = "page-nonce";
	original.setAttribute("disabled", "");

	await NEXT_HMR();
	const replacement = document.querySelector("link");
	expect(replacement.href).toContain("bundle.css?hmr=");
	for (const [name, value] of Object.entries(attributes)) {
		expect(replacement.getAttribute(name)).toBe(value);
	}
	for (const name of discarded) {
		expect(replacement.hasAttribute(name)).toBe(false);
	}
	expect(replacement.nonce).toBe("page-nonce");
	expect(replacement.hasAttribute("disabled")).toBe(true);

	// CSSOM can disable a stylesheet without a disabled HTML attribute.
	replacement.removeAttribute("disabled");
	replacement.sheet.disabled = true;
	__webpack_nonce__ = "runtime-nonce";
	await NEXT_HMR();
	const disabled = document.querySelector("link");
	expect(disabled.hasAttribute("disabled")).toBe(false);
	expect(disabled.sheet.disabled).toBe(true);
	expect(disabled.nonce).toBe("runtime-nonce");

	disabled.sheet.disabled = false;
	await NEXT_HMR();
	expect(document.querySelector("link").sheet.disabled).toBe(false);
});

module.hot.accept("./index.css");
