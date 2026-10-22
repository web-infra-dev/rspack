import fs from "fs";
import path from "path";
import "./style.css";

it("resolves an unterminated url() at end-of-input without dropping its last byte", () => {
	// If the tokenizer dropped the final byte, `./img.png` would become
	// `./img.pn` and fail to resolve, breaking the build before this runs.
	// If it treated the url as bad, the request would be left unresolved.
	const css = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
	expect(css).not.toContain("./img.png");
	const [, asset] = css.match(/url\("?([^")]+\.png)"?\)/);
	expect(fs.existsSync(path.resolve(__dirname, asset))).toBe(true);
});
