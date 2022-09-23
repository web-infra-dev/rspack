function getCurrentScriptSource(): string {
	if (document.currentScript) {
		return document.currentScript.getAttribute("src");
	}

	const scriptElements = document.scripts || [];
	const scriptElementsWithSrc = Array.prototype.filter.call(
		scriptElements,
		(element: HTMLElement) => element.getAttribute("src")
	);

	if (scriptElementsWithSrc.length > 0) {
		const currentScript =
			scriptElementsWithSrc[scriptElementsWithSrc.length - 1];

		return currentScript.getAttribute("src");
	}

	throw new Error("Failed to get current script source.");
}

export default getCurrentScriptSource;
