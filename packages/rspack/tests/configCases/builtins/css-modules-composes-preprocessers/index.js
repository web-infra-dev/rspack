it("css modules with css preprocessers", () => {
	const style = require("./index.css");
	expect(style).toEqual({
		class: "class-index.css lessClass-less-file.less ",
		ghi: "ghi-index.css ",
		other: "other-index.css scssClass-scss-file.scss ",
		otherClassName: "otherClassName-index.css globalClassName "
	});
});
