import "./style.css";

it("should preserve the public path returned by output.publicPath in extracted CSS", () => {
	const css = require("fs").readFileSync(`${__dirname}/style.css`, "utf-8");
	expect(css).toContain("url(../assets/file.png)");
});
