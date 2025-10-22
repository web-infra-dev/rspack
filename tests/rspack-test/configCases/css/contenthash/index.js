import * as style from "./style.css";

it("should work with js", async () => {
	await import('./async.js').then(x => {
		expect(x.name).toBe("async")
	});
});

it("should work with css", async () => {
	expect(style).toEqual(nsObj({}));

	const computedStyle = getComputedStyle(document.body);

	expect(computedStyle.getPropertyValue("background")).toBe("green");
	expect(computedStyle.getPropertyValue("color")).toBe("rgb(255, 255, 0)");

	await import("./async.css").then(x => {
		expect(x).toEqual(nsObj({}));

		const style = getComputedStyle(document.body);

		expect(style.getPropertyValue("background")).toBe("yellow");
		expect(style.getPropertyValue("color")).toBe("rgb(0, 128, 0)");
	});
});
