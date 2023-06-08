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
  a: url("82ee8285df64be76.svg");
  b: url("82ee8285df64be76.svg");
  c: url("82ee8285df64be76");
  d: url("82ee8285df64be76");
}`);
	expect(inlineSvg).toBe(
		'data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg"></svg>'
	);
});
