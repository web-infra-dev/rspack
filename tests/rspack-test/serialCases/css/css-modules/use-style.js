import * as style from "./style.module.css";
import { local1, local2, local3, local4, ident } from "./style.module.css";

export default {
	// TODO: enable when missing CSS module exports can be reported as warnings.
	// global: style.global,
	class: style.class,
	local: `${local1} ${local2} ${local3} ${local4}`,
	local2: `${style.local5} ${style.local6}`,
	// TODO: include style.nested2 when missing CSS module exports can be reported as warnings.
	nested: `${style.nested1} ${style.nested3}`,
	ident,
	keyframes: style.localkeyframes,
	animation: style.animation,
	// TODO: include local-color/global-color when CSS custom property exports are supported.
	vars: `${style.vars} ${style.globalVars}`
};
