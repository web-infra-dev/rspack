it("css modules with css preprocessers", () => {
	const style = require("./index.css");
	expect(style).toEqual({
		class: "-index-css__class -less-file-less__lessClass ",
		ghi: "-index-css__ghi ",
		other: "-index-css__other -scss-file-scss__scssClass ",
		otherClassName: "-index-css__otherClassName globalClassName "
	});
});
