import { TextDecoder, TextEncoder } from "node:util";

const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8");

function utf8BufferToString(buf: Uint8Array) {
	const str = decoder.decode(buf);
	if (str.charCodeAt(0) === 0xfeff) {
		return str.slice(1);
	}
	return str;
}

export function convertArgs(args: any[], raw: boolean) {
	if (!raw && args[0] instanceof Uint8Array)
		args[0] = utf8BufferToString(args[0]);
	else if (raw && typeof args[0] === "string")
		args[0] = encoder.encode(args[0]);
}
