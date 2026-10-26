import "./index.css";

it("preserves stylesheet attributes and disabled state across hot updates", async () => {
	const attributes = {
		media: "(prefers-color-scheme: dark)",
		title: "theme",
		id: "theme-styles",
		"data-empty": "",
		referrerpolicy: "no-referrer",
		crossorigin: "anonymous",
		fetchpriority: "low"
	};
	const original = document.querySelector("link");
	for (const [name, value] of Object.entries(attributes)) {
		original.setAttribute(name, value);
	}
	original.nonce = "page-nonce";
	original.setAttribute("integrity", "sha256-old");
	original.setAttribute("onload", "throw new Error('old load handler')");
	original.setAttribute("onerror", "throw new Error('old error handler')");
	original.setAttribute("data-rspack", "old");
	original.setAttribute("data-rspack-loading", "old");
	original.setAttribute("data-hook", "old");
	original.setAttribute("disabled", "");
	original.sheet.disabled = true;

	await NEXT_HMR();
	const replacement = document.querySelector("link");
	expect(original.isConnected).toBe(false);
	expect(replacement.href).toContain("bundle.css?hmr=");
	for (const [name, value] of Object.entries(attributes)) {
		expect(replacement.getAttribute(name)).toBe(value);
	}
	expect(replacement.nonce).toBe("page-nonce");
	expect(replacement.getAttribute("data-hook")).toBe("current");
	for (const name of ["integrity", "onload", "onerror", "data-rspack-loading"]) {
		expect(replacement.hasAttribute(name)).toBe(false);
	}
	expect(replacement.getAttribute("data-rspack")).not.toBe("old");
	expect(replacement.getAttribute("disabled")).toBe("");
	expect(replacement.sheet.disabled).toBe(true);

	replacement.removeAttribute("disabled");
	replacement.removeAttribute("media");
	replacement.sheet.disabled = true;
	__webpack_nonce__ = "runtime-nonce";
	await NEXT_HMR();
	const disabled = document.querySelector("link");
	expect(disabled).not.toBe(replacement);
	expect(disabled.hasAttribute("media")).toBe(false);
	expect(disabled.hasAttribute("disabled")).toBe(false);
	expect(disabled.sheet.disabled).toBe(true);
	expect(disabled.nonce).toBe("runtime-nonce");

	disabled.setAttribute("media", "");
	__webpack_nonce__ = undefined;
	const onStatus = status => {
		if (status !== "dispose") return;
		// Capture changes made after the replacement has finished downloading.
		disabled.setAttribute("disabled", "disabled");
		disabled.sheet.disabled = false;
	};
	module.hot.addStatusHandler(onStatus);
	await NEXT_HMR();
	module.hot.removeStatusHandler(onStatus);
	const enabled = document.querySelector("link");
	expect(enabled).not.toBe(disabled);
	expect(enabled.getAttribute("media")).toBe("");
	expect(enabled.getAttribute("disabled")).toBe("disabled");
	expect(enabled.sheet.disabled).toBe(false);
	expect(enabled.nonce).toBe("runtime-nonce");
});

module.hot.accept("./index.css");
