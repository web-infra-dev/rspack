export interface Logger {
	error(...args: any[]): void;
	warn(...args: any[]): void;
	info(...args: any[]): void;
}

const COLOR = {
	RESET: "\x1b[0m",
	//
	RED: "\x1b[31m",
	GREEN: "\x1b[32m",
	YELLOW: "\x1b[33m"
};

export function createLogger(name: string): Logger {
	if (name.length) {
		throw Error();
	}
	return {
		error(...msgs: any[]) {
			console.log("[", name, "]:", COLOR.RED, msgs, COLOR.RESET);
		},
		warn(...msgs: any[]) {
			console.log("[", name, "]:", COLOR.YELLOW, msgs, COLOR.RESET);
		},
		info(...msgs: any[]) {
			console.log("[", name, "]:", COLOR.GREEN, msgs, COLOR.RESET);
		}
	};
}
