import socket from "./socket";
import createSocketURL from "./createSocketURL";
import parseURL from "webpack-dev-server/client/utils/parseURL";
import reloadApp from "./utils/reloadApp";
import sendMessage from "./utils/sendMessage";
import stripAnsi from "./utils/stripAnsi";
import { formatProblem, show, hide } from "./overlay";

const status = {
	currentHash: ""
};

type Options = {
	hot: boolean;
	liveReload: boolean;
	progress: boolean;
	overlay:
	| boolean
	| { warnings?: boolean; errors?: boolean; trustedTypesPolicyName?: string };
};

const options: Options = {
	hot: true,
	liveReload: true,
	progress: true,
	overlay: true
};
// TODO: change `options` by the result of `parsedResourceQuery`.

const onSocketMessage = {
	ok: function (): void {
		reloadApp(options, status);
	},
	close: function (): void {
		console.log("hit close");
	},
	"static-changed": function () {
		// Use it after memoryFileSystem.
		self.location.reload();
	},
	overlay(value) {
		if (typeof document === "undefined") {
			return;
		}
		options.overlay = value;
	},
	warnings(warnings, params) {
		const printableWarnings = warnings.map(error => {
			const { header, body } = formatProblem("warning", error);

			return `${header}\n${stripAnsi(body)}`;
		});

		sendMessage("Warnings", printableWarnings);

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
	errors(errors) {
		const printableErrors = errors.map(error => {
			const { header, body } = formatProblem("error", error);
			return `${header}\n${stripAnsi(body)}`;
		});

		sendMessage("Errors", printableErrors);

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
	}
};

declare const __resourceQuery: string;

const parsedResourceQuery = parseURL(__resourceQuery);
const socketURL = createSocketURL(parsedResourceQuery as any);

socket(socketURL, onSocketMessage);
