import type * as binding from "@rspack/binding";

export type RspackError = binding.JsRspackError;
export type RspackSeverity = binding.JsRspackSeverity;

export class NonErrorEmittedError extends Error {
	constructor(error: Error) {
		super();
		this.name = "NonErrorEmittedError";
		this.message = `(Emitted value instead of an instance of Error) ${error}`;
	}
}

export class DeadlockRiskError extends Error {
	constructor(message: string) {
		super(message);
		this.name = "DeadlockRiskError";
		// hide the stack trace for this error
		this.stack = "";
	}
}
