import "./style.css";

it("should compile with warning", () => {
	const style = getComputedStyle(document.body);
	expect(style.getPropertyValue("background")).toBe(" red");
});
