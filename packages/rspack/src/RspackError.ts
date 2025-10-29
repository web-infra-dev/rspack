import type binding from "@rspack/binding";

export type { RspackError } from "@rspack/binding";
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

// This type is no longer used and will be removed in Rspack 2.0
export class ValidationError extends Error {
	constructor(message: string) {
		super(message);
		this.name = "ValidationError";
	}
}
