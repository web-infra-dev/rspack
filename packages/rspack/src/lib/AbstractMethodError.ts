/*
	MIT License http://www.opensource.org/licenses/mit-license.php
	Author Ivan Kopeykin @vankop
*/

import { WebpackError } from "./WebpackError";
const CURRENT_METHOD_REGEXP = /at ([a-zA-Z0-9_.]*)/;

/**
 * @param {string=} method method name
 * @returns {string} message
 */
function createMessage(method?: string): string {
	return `Abstract method${method ? " " + method : ""}. Must be overridden.`;
}

class Message {
	stack: string | undefined;
	message: string | undefined;
	constructor() {
		this.stack = undefined;
		Error.captureStackTrace(this);
		const match = (this.stack as unknown as string).split("\n")[3].match(CURRENT_METHOD_REGEXP);

		this.message = match && match[1] ? createMessage(match[1]) : createMessage();
	}

}


/**
 * Error for abstract method
 * @example
 * class FooClass {
 *     abstractMethod() {
 *         throw new AbstractMethodError(); // error message: Abstract method FooClass.abstractMethod. Must be overridden.
 *     }
 * }
 *
 */
class AbstractMethodError extends WebpackError {
	constructor() {
		super(new Message().message as string);
		this.name = "AbstractMethodError";
	}
}

export { AbstractMethodError }
