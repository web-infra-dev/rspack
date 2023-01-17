/*
	The following code is modified from https://github.com/webpack/webpack/blob/main/lib/ConcurrentCompilationError.js
	MIT License http://www.opensource.org/licenses/mit-license.php
	Author Maksim Nazarjev @acupofspirt
*/

export default class ConcurrentCompilationError extends Error {
	name: string;
	message: string;

	constructor() {
		super();
		this.name = "ConcurrentCompilationError";
		this.message =
			"You ran Webpack twice. Each instance only supports a single concurrent compilation at a time.";
	}
}
