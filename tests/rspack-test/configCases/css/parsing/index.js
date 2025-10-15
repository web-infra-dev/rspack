import "./style.css";

it("should compile and load style on demand", () => {
	const style = getComputedStyle(document.body);
	expect(style.getPropertyValue("background")).toBe(" red");
});
