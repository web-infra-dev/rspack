import "./style.css";
import value from "./value";

function stylesheets() {
	return Array.from(document.getElementsByTagName("link")).filter(function (link) {
		return ((link.getAttribute("href") || "").split("?")[0]).endsWith("main.css");
	});
}

it("should keep the stylesheet untouched when only js changes", async () => {
	expect(value).toBe(1);
	let before;
	if (typeof document !== "undefined") {
		const links = stylesheets();
		expect(links.length).toBe(1);
		before = links[0];
	}
	await NEXT_HMR();
	expect(require("./value").default).toBe(2);
	if (typeof document !== "undefined") {
		const links = stylesheets();
		expect(links.length).toBe(1);
		expect(links[0]).toBe(before);
		expect((links[0].getAttribute("href") || "").indexOf("?")).toBe(-1);
	}
});

module.hot.accept("./value");
