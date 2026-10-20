import value from "./app";

it("should preserve the original stylesheet node through consecutive updates", async () => {
	if (typeof document === "undefined") return;
	const original = Array.from(document.querySelectorAll("link")).find((link) =>
		link.href.split("?")[0].endsWith("main.css"),
	);
	expect(original).toBeDefined();
	const parent = original.parentNode;
	let href = original.href;
	for (let step = 0; step < 2; step++) {
		await NEXT_HMR();
		const deadline = Date.now() + 5000;
		while (
			(original.href === href ||
				document.querySelectorAll("link").length !== 1) &&
			Date.now() < deadline
		) {
			await new Promise((resolve) => setTimeout(resolve, 10));
		}
		expect(original.parentNode).toBe(parent);
		expect(original.disabled).not.toBe(true);
		expect(original.href).not.toBe(href);
		expect(document.querySelectorAll("link")).toHaveLength(1);
		expect(value).toBe(step + 2);
		href = original.href;
	}
	parent.removeChild(original);
	expect(original.parentNode).toBe(null);
});

module.hot.accept("./app");
