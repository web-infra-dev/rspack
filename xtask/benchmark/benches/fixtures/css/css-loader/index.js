// Copied from tests/rspack-test/configCases/css/css-loader/index.js.
// Keep the module imports; expose every namespace instead of running test assertions.
import * as basic from "./basic.module.css";
import * as styles from "./classes.module.css";
import * as styles1 from "./composes-multiple.module.css";
import * as styles3 from "./composes-global.module.css";
import * as styles4 from "./scope-at-rule.module.css";
import * as styles5 from "./nesting.module.css";
import * as styles6 from "./prefer-relative.module.css";
import * as styles7 from "./animation-name.module.css";
import * as styles8 from "./at-sign-in-package-name.module.css";
import * as styles9 from "./resolving-from-node_modules.module.css";
import * as styles10 from "./local-Ident-name.module.css";
import * as styles11 from "./local-Ident-name.module.css?local-ident-name-1";
import * as styles12 from "./local-Ident-name.module.css?local-ident-name-2";
import * as styles13 from "./local-Ident-name.module.css?local-ident-name-3";
import * as styles14 from "./local-Ident-name.module.css?local-ident-name-4";
import * as styles15 from "./local-Ident-name.module.css?local-ident-name-5";
import * as styles16 from "./local-Ident-name.module.css?local-ident-name-6";
import * as styles17 from "./local-Ident-name.module.css?local-ident-name-7";
import * as styles18 from "./local-Ident-name.module.css?local-ident-name-8";
import * as styles19 from "./local-Ident-name.module.css?local-ident-name-9";
import * as stylesHash10 from "./local-Ident-name.module.css?local-ident-name-10";
import * as stylesHash11 from "./local-Ident-name.module.css?local-ident-name-11";
import * as stylesHash12 from "./local-Ident-name.module.css?local-ident-name-12";
import * as stylesHash13 from "./local-Ident-name.module.css?local-ident-name-13";
import * as stylesHash14 from "./local-Ident-name.module.css?local-ident-name-14";
import * as styles20 from "./order.module.css";
import * as styles21 from "./dedup.module.css";
import * as styles22 from "./composes-from-less.module.css";
import * as styles23 from "./tilde.module.css";
import * as styles24 from "./icss.module.css";
import * as styles25 from "./empty.module.css";
import * as styles26 from "./component-name.module.css";
import * as styles27 from "./composes-chain.module.css";
import * as styles28 from "./file.with.many.dots.in.name.module.css";
import * as styles29 from "./composes-duplicate.module.css";
import * as styles30 from "./keyframes-leak-scope.module.css";
import * as styles31 from "./path-placeholder.module.css";
import * as styles32 from "./at-value-extra.module.css";
import * as styles33 from "./composes-circular.module.css";

globalThis.__rspackCssLoader = [
	basic,
	styles,
	styles1,
	styles3,
	styles4,
	styles5,
	styles6,
	styles7,
	styles8,
	styles9,
	styles10,
	styles11,
	styles12,
	styles13,
	styles14,
	styles15,
	styles16,
	styles17,
	styles18,
	styles19,
	stylesHash10,
	stylesHash11,
	stylesHash12,
	stylesHash13,
	stylesHash14,
	styles20,
	styles21,
	styles22,
	styles23,
	styles24,
	styles25,
	styles26,
	styles27,
	styles28,
	styles29,
	styles30,
	styles31,
	styles32,
	styles33,
];
