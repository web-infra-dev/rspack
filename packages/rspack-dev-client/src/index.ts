import webpackHotLog from "webpack/hot/log.js";
import socket from "./socket";
import createSocketURL from "webpack-dev-server/client/utils/createSocketURL";
import parseURL from "webpack-dev-server/client/utils/parseURL";
// WARNING: reloadApp will import a instance of emitter from webpack/hot.
// If a different instance is referenced, it can cause hmr to fail
import reloadApp from "webpack-dev-server/client/utils/reloadApp";
import sendMessage from "webpack-dev-server/client/utils/sendMessage";
import stripAnsi from "webpack-dev-server/client/utils/stripAnsi";
import { formatProblem, show, hide } from "webpack-dev-server/client/overlay";
import { log, setLogLevel } from "webpack-dev-server/client/utils/log";

declare const __resourceQuery: string;
declare const __webpack_hash__: string;

function setAllLogLevel(level) {
	webpackHotLog.setLogLevel(
		level === "verbose" || level === "log" ? "info" : level
	);
	setLogLevel(level);
}

const status = {
	isUnloading: false,
	currentHash: typeof __webpack_hash__ !== "undefined" ? __webpack_hash__ : "",
	previousHash: undefined
};

type Options = {
	hot: boolean;
	liveReload: boolean;
	progress: boolean;
	overlay:
		| boolean
		| { warnings?: boolean; errors?: boolean; trustedTypesPolicyName?: string };
	reconnect: number;
};

const options: Options = {
	hot: true,
	liveReload: true,
	progress: true,
	overlay: true,
	reconnect: 3
};
// TODO: change `options` by the result of `parsedResourceQuery`.

const onSocketMessage = {
	hot() {
		if (parsedResourceQuery.hot === "false") {
			return;
		}

		options.hot = true;
	},
	liveReload() {
		if (parsedResourceQuery["live-reload"] === "false") {
			return;
		}

		options.liveReload = true;
	},
	invalid() {
		log.info("App updated. Recompiling...");

		// Fixes #1042. overlay doesn't clear if errors are fixed but warnings remain.
		if (options.overlay) {
			hide();
		}

		sendMessage("Invalid");
	},
	/**
	 * @param {string} hash
	 */
	hash(hash) {
		status.previousHash = status.currentHash;
		status.currentHash = hash;
	},
	logging: setAllLogLevel,
	/**
	 * @param {boolean} value
	 */
	overlay(value) {
		if (typeof document === "undefined") {
			return;
		}

		options.overlay = value;
	},
	/**
	 * @param {number} value
	 */
	reconnect(value) {
		if (parsedResourceQuery.reconnect === "false") {
			return;
		}

		options.reconnect = value;
	},
	/**
	 * @param {boolean} value
	 */
	progress(value) {
		options.progress = value;
	},
	/**
	 * @param {{ pluginName?: string, percent: number, msg: string }} data
	 */
	"progress-update": function progressUpdate(data) {
		if (options.progress) {
			log.info(
				`${data.pluginName ? `[${data.pluginName}] ` : ""}${data.percent}% - ${
					data.msg
				}.`
			);
		}

		sendMessage("Progress", data);
	},
	"still-ok": function stillOk() {
		log.info("Nothing changed.");

		if (options.overlay) {
			hide();
		}

		sendMessage("StillOk");
	},
	ok() {
		sendMessage("Ok");

		if (options.overlay) {
			hide();
		}

		reloadApp(options, status);
	},
	// TODO: remove in v5 in favor of 'static-changed'
	/**
	 * @param {string} file
	 */
	"content-changed": function contentChanged(file) {
		log.info(
			`${
				file ? `"${file}"` : "Content"
			} from static directory was changed. Reloading...`
		);

		self.location.reload();
	},
	/**
	 * @param {string} file
	 */
	"static-changed": function staticChanged(file) {
		log.info(
			`${
				file ? `"${file}"` : "Content"
			} from static directory was changed. Reloading...`
		);

		self.location.reload();
	},
	/**
	 * @param {Error[]} warnings
	 * @param {any} params
	 */
	warnings(warnings, params) {
		log.warn("Warnings while compiling.");

		const printableWarnings = warnings.map(error => {
			const { header, body } = formatProblem("warning", error);

			return `${header}\n${stripAnsi(body)}`;
		});

		sendMessage("Warnings", printableWarnings);

		for (let i = 0; i < printableWarnings.length; i++) {
			log.warn(printableWarnings[i]);
		}

		const needShowOverlayForWarnings =
			typeof options.overlay === "boolean"
				? options.overlay
				: options.overlay && options.overlay.warnings;

		if (needShowOverlayForWarnings) {
			const trustedTypesPolicyName =
				typeof options.overlay === "object" &&
				options.overlay.trustedTypesPolicyName;
			show("warning", warnings, trustedTypesPolicyName || null);
		}

		if (params && params.preventReloading) {
			return;
		}

		reloadApp(options, status);
	},
	/**
	 * @param {Error[]} errors
	 */
	errors(errors) {
		log.error("Errors while compiling. Reload prevented.");

		const printableErrors = errors.map(error => {
			const { header, body } = formatProblem("error", error);

			return `${header}\n${stripAnsi(body)}`;
		});

		sendMessage("Errors", printableErrors);

		for (let i = 0; i < printableErrors.length; i++) {
			log.error(printableErrors[i]);
		}

		const needShowOverlayForErrors =
			typeof options.overlay === "boolean"
				? options.overlay
				: options.overlay && options.overlay.errors;

		if (needShowOverlayForErrors) {
			const trustedTypesPolicyName =
				typeof options.overlay === "object" &&
				options.overlay.trustedTypesPolicyName;
			show("error", errors, trustedTypesPolicyName || null);
		}
	},
	error(error) {
		log.error(error);
	},
	close() {
		log.info("Disconnected!");

		if (options.overlay) {
			hide();
		}

		sendMessage("Close");
	}
};

const parsedResourceQuery = parseURL(__resourceQuery);
const socketURL = createSocketURL(parsedResourceQuery);

socket(socketURL, onSocketMessage);
