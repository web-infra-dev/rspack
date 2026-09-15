import { foo } from './index.module.css'

const fs = require("fs");
const path = require("path");

it("should generate correct css", () => {
	expect(foo).toBe("./index.module-foo bar")
	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	expect(css).not.toContain("module-baz")
	expect(css).toContain(".module-foo")
	expect(css).toContain(".module-bar")
	expect(css).toContain("@keyframes")
})
