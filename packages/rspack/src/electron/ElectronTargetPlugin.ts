/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/electron/ElectronTargetPlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import { Compiler } from "..";
import { ExternalsPlugin } from "../builtin-plugin";

export default class ElectronTargetPlugin {
	constructor(private _context?: "main" | "preload" | "renderer") {}

	apply(compiler: Compiler) {
		new ExternalsPlugin("node-commonjs", [
			"clipboard",
			"crash-reporter",
			"electron",
			"ipc",
			"native-image",
			"original-fs",
			"screen",
			"shell"
		]).apply(compiler);
		switch (this._context) {
			case "main":
				new ExternalsPlugin("node-commonjs", [
					"app",
					"auto-updater",
					"browser-window",
					"content-tracing",
					"dialog",
					"global-shortcut",
					"ipc-main",
					"menu",
					"menu-item",
					"power-monitor",
					"power-save-blocker",
					"protocol",
					"session",
					"tray",
					"web-contents"
				]).apply(compiler);
				break;
			case "preload":
			case "renderer":
				new ExternalsPlugin("node-commonjs", [
					"desktop-capturer",
					"ipc-renderer",
					"remote",
					"web-frame"
				]).apply(compiler);
				break;
		}
	}
}
