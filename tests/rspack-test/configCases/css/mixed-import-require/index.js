import "./style1.css"; // red
require("./style2.css"); // blue

it("should place style2 before style1", () => {
	const style = getComputedStyle(document.body);
	// should be blue
	expect(style.getPropertyValue("color")).toBe("rgb(0, 0, 255)");
});
