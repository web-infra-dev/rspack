import * as styles from "./style.modules.css";
import * as globalStyles from "./style.global.css";
import * as smallStyles from "./small.modules.css";
import * as syntaxStyles from "./syntax.modules.css";
import * as recoveryStyles from "./recovery.modules.css";
import * as pureRecoveryStyles from "./recovery.pure.modules.css";
import * as eofStyles from "./eof.modules.css";
import * as eofCurlyStyles from "./eof-curly.modules.css";
import "./recovery.css";

it("localizes type-first selectors in nested rules", () => {
	const fs = require("fs");
	const path = require("path");
	const css = fs.readFileSync(path.join(__dirname, "bundle0.css"), "utf-8");

	expect(css).toMatchFileSnapshotSync(
		path.join(__SNAPSHOT__, "bundle0.css.txt")
	);
	expect({
		local: styles,
		global: globalStyles,
		short: smallStyles,
		syntax: syntaxStyles,
		recovery: recoveryStyles,
		pureRecovery: pureRecoveryStyles,
		eof: eofStyles,
		eofCurly: eofCurlyStyles
	}).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "exports.txt"));
});
