import { btn, mixed } from "./style.module.css";
import { shared } from "./shared.module.css";
import { target } from "./target.module.css";

it("should share CSS imports, compositions and JS imports with the same export type", () => {
	expect(shared).toBe("shared");
	expect(btn).toBe("btn shared");
	expect(target).toBe("target");
	expect(mixed).toBe("mixed target");
	if (EXPORT_TYPE === "style") {
		const sharedStyles = Array.from(document.getElementsByTagName("style"))
			.filter(style => style.textContent.includes("color: red"));
		expect(sharedStyles.length).toBe(1);
	}
});
