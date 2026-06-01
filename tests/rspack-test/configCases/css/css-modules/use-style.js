// TODO: enable the full style.module.css assertions once the remaining
// webpack css-modules selector fixture no longer panics css-module-lexer.
// import * as style from "./style.module.css";
// import { local1, local2, local3, local4, ident } from "./style.module.css";
import { myCssClass } from "./style.module.my-css";
import * as notACssModule from "./style.module.css.invalid";
import { UsedClassName } from "./identifiers.module.css";

// To prevent analysis export
const isNotACSSModule = typeof notACssModule["c" + "lass"] === "undefined";
const hasOwnProperty = (obj, p) => Object.hasOwnProperty.call(obj, p);

export default {
	cssModuleWithCustomFileExtension: myCssClass,
	notAValidCssModuleExtension: isNotACSSModule,
	UsedClassName,
	exportLocalVarsShouldCleanup: `${hasOwnProperty(notACssModule, 'local-color')} ${hasOwnProperty(notACssModule, "LOCAL-COLOR")}`
};
