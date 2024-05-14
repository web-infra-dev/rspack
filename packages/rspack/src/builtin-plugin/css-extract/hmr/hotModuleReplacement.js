/* eslint-env browser */
/*
  eslint-disable
  no-console,
  func-names
*/

/** @typedef {any} TODO */

const normalizeUrl = require("./normalize-url");

const srcByModuleId = Object.create(null);

const noDocument = typeof document === "undefined";

const { forEach } = Array.prototype;

/**
 * @param {function} fn
 * @param {number} time
 * @returns {(function(): void)|*}
 */
function debounce(fn, time) {
	let timeout = 0;

	return function () {
		// @ts-ignore
		const self = this;
		// eslint-disable-next-line prefer-rest-params
		const args = arguments;

		const functionCall = function functionCall() {
			return fn.apply(self, args);
		};

		clearTimeout(timeout);

		// @ts-ignore
		timeout = setTimeout(functionCall, time);
	};
}

function noop() {}

/**
 * @param {TODO} moduleId
 * @returns {TODO}
 */
function getCurrentScriptUrl(moduleId) {
	let src = srcByModuleId[moduleId];

	if (!src) {
		if (document.currentScript) {
			({ src } = /** @type {HTMLScriptElement} */ (document.currentScript));
		} else {
			const scripts = document.getElementsByTagName("script");
			const lastScriptTag = scripts[scripts.length - 1];

			if (lastScriptTag) {
				({ src } = lastScriptTag);
			}
		}

		srcByModuleId[moduleId] = src;
	}

	/**
	 * @param {string} fileMap
	 * @returns {null | string[]}
	 */
	return function (fileMap) {
		if (!src) {
			return null;
		}

		const splitResult = src.split(/([^\\/]+)\.js$/);
		const filename = splitResult && splitResult[1];

		if (!filename) {
			return [src.replace(".js", ".css")];
		}

		if (!fileMap) {
			return [src.replace(".js", ".css")];
		}

		return fileMap.split(",").map(mapRule => {
			const reg = new RegExp(`${filename}\\.js$`, "g");

			return normalizeUrl(
				src.replace(reg, `${mapRule.replace(/{fileName}/g, filename)}.css`)
			);
		});
	};
}

/**
 * @param {TODO} el
 * @param {string} [url]
 */
function updateCss(el, url) {
	if (!url) {
		if (!el.href) {
			return;
		}

		// eslint-disable-next-line
		url = el.href.split("?")[0];
	}

	if (!isUrlRequest(/** @type {string} */ (url))) {
		return;
	}

	if (el.isLoaded === false) {
		// We seem to be about to replace a css link that hasn't loaded yet.
		// We're probably changing the same file more than once.
		return;
	}

	if (!url || !(url.indexOf(".css") > -1)) {
		return;
	}

	// eslint-disable-next-line no-param-reassign
	el.visited = true;

	const newEl = el.cloneNode();

	newEl.isLoaded = false;

	newEl.addEventListener("load", () => {
		if (newEl.isLoaded) {
			return;
		}

		newEl.isLoaded = true;
		el.parentNode.removeChild(el);
	});

	newEl.addEventListener("error", () => {
		if (newEl.isLoaded) {
			return;
		}

		newEl.isLoaded = true;
		el.parentNode.removeChild(el);
	});

	newEl.href = `${url}?${Date.now()}`;

	if (el.nextSibling) {
		el.parentNode.insertBefore(newEl, el.nextSibling);
	} else {
		el.parentNode.appendChild(newEl);
	}
}

/**
 * @param {string} href
 * @param {TODO} src
 * @returns {TODO}
 */
function getReloadUrl(href, src) {
	let ret;

	// eslint-disable-next-line no-param-reassign
	href = normalizeUrl(href);

	src.some(
		/**
		 * @param {string} url
		 */
		// eslint-disable-next-line array-callback-return
		url => {
			if (href.indexOf(src) > -1) {
				ret = url;
			}
		}
	);

	return ret;
}

/**
 * @param {string} [src]
 * @returns {boolean}
 */
function reloadStyle(src) {
	if (!src) {
		return false;
	}

	const elements = document.querySelectorAll("link");
	let loaded = false;

	forEach.call(elements, el => {
		if (!el.href) {
			return;
		}

		const url = getReloadUrl(el.href, src);

		if (!isUrlRequest(url)) {
			return;
		}

		if (el.visited === true) {
			return;
		}

		if (url) {
			updateCss(el, url);

			loaded = true;
		}
	});

	return loaded;
}

function reloadAll() {
	const elements = document.querySelectorAll("link");

	forEach.call(elements, el => {
		if (el.visited === true) {
			return;
		}

		updateCss(el);
	});
}

/**
 * @param {string} url
 * @returns {boolean}
 */
function isUrlRequest(url) {
	// An URL is not an request if

	// It is not http or https
	if (!/^[a-zA-Z][a-zA-Z\d+\-.]*:/.test(url)) {
		return false;
	}

	return true;
}

/**
 * @param {TODO} moduleId
 * @param {TODO} options
 * @returns {TODO}
 */
module.exports = function (moduleId, options) {
	if (noDocument) {
		console.log("no window.document found, will not HMR CSS");

		return noop;
	}

	const getScriptSrc = getCurrentScriptUrl(moduleId);

	function update() {
		const src = getScriptSrc(options.filename);
		const reloaded = reloadStyle(src);

		if (options.locals) {
			console.log("[HMR] Detected local css modules. Reload all css");

			reloadAll();

			return;
		}

		if (reloaded) {
			console.log("[HMR] css reload %s", src.join(" "));
		} else {
			console.log("[HMR] Reload all css");

			reloadAll();
		}
	}

	return debounce(update, 50);
};
