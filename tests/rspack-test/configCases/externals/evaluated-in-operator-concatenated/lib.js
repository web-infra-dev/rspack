import nodeUtil from "node:util";
import nodeProcess from "node:process";
import nodeOs from "node:os";
import nodeTty from "node:tty";

function checkNodeVersion() {
	const { versions } = process;
	if ("styleText" in nodeUtil || !versions.node || versions.bun || versions.deno) {
		return;
	}
	throw new Error(
		`Unsupported Node.js version: "${process.versions.node || "unknown"}". Expected Node.js >= 20.`
	);
}

checkNodeVersion();

export const colorize = (text) => nodeUtil.styleText("blue", String(text));

export function supportsBasicColor() {
	if ("FORCE_COLOR" in nodeProcess.env) {
		return true;
	}

	if (nodeProcess.platform === "win32") {
		return nodeOs.release().length > 0;
	}

	return Boolean(nodeTty.isatty?.(1));
}
