import { component, dependency, dependency2 } from "./component";
component(dependency, dependency2);

// https://github.com/webpack/webpack/issues/18961
// https://github.com/jantimon/reproduction-webpack-css-order
it("keep consistent css order", function () {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");
	let source = fs.readFileSync(__dirname + "/main.css", "utf-8");
	expect(removeComments(source)).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, 'main.css.txt'))
});

function removeComments(source) {
	return source.replace(/\/\*[\s\S]*?\*\//g, "").replace(/\n/g, "");
}
