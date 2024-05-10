// @ts-nocheck
/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/test/helpers/EventSourceForNode.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

"use strict";

module.exports = class EventSource {
	constructor(url) {
		this.response = undefined;
		const request = (
			url.startsWith("https:") ? require("https") : require("http")
		).request(
			url,
			{
				agent: false,
				headers: { accept: "text/event-stream" }
			},
			res => {
				this.response = res;
				res.on("error", err => {
					if (this.onerror) this.onerror(err);
				});
			}
		);
		request.on("error", err => {
			if (this.onerror) this.onerror({ message: err });
		});
		request.end();
	}

	close() {
		this.response.destroy();
	}

	set onopen(value) {
		throw new Error("not implemented");
	}

	set onmessage(value) {
		throw new Error("not implemented");
	}
};
