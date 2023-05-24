import a from 'data:text/javascript,export default "a";';
import "data:text/css,.red{color: red;}";
import "./index.css";
import inlineSvg from 'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg"></svg>';
import fs from "node:fs";
import path from "node:path";

it("data imports", () => {
	expect(a).toBe("a");
	expect(fs.readFileSync(path.resolve(__dirname, "main.css"), "utf-8"))
		.toBe(`.red {
  color: red;
}

.b {
  color: green;
}

.a {
  color: palegreen;
}
;;;


.class {
  a: url("2c6053f86393fdda.svg");
  b: url("2c6053f86393fdda.svg");
  c: url("2c6053f86393fdda");
  d: url("2c6053f86393fdda");
}`);
	expect(inlineSvg).toBe(
		'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg"></svg>'
	);
});
