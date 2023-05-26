const fs = require("fs");
const path = require("path");

it("css modules exportsOnly", () => {
	const style = require("./index.css");
	expect(fs.existsSync(path.resolve(__dirname, "./main.css"))).toBe(false);
	expect(style).toEqual({
		"simple-bar": "index-css__simple-bar imported-simple-css__imported-simple",
		"simple-foo": "index-css__simple-foo imported-simple-css__imported-simple"
	});
});
