const fs = require("fs")
const path = require("path")
it("should keep the extracted license file stable", () => {
	require("foo")
	require("bar")
	require("baz")
	require("./relative")
	expect(fs.readFileSync(path.join(__dirname, "bundle0.js.LICENSE.txt"), "utf8")).toMatchSnapshot()
})
