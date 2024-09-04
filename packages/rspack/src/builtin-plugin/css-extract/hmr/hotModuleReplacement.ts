import { normalizeUrl } from "./normalizeUrl";

type Option<T> = T | null | undefined;
type DebouncedFunction<T extends (...args: any[]) => any> = (
	...args: Parameters<T>
) => void;

const srcByModuleId: Record<string, any> = Object.create(null);

const noDocument = typeof document === "undefined";

const { forEach } = Array.prototype;

function debounce<T extends (...args: any[]) => any>(
	fn: T,
	time: number
): DebouncedFunction<T> {
	let timeout: NodeJS.Timeout | number = 0;

	return function (this: any, ...args: Parameters<T>[]) {
		const self = this;

		const functionCall = function functionCall() {
			return fn.apply(self, args as Parameters<T>);
		};

		clearTimeout(timeout);

		timeout = setTimeout(functionCall, time);
	};
}

function noop() {}

function getCurrentScriptUrl(moduleId: string) {
	let src = srcByModuleId[moduleId];

	if (!src) {
		if (document.currentScript) {
			({ src } = document.currentScript as HTMLScriptElement);
		} else {
			const scripts = document.getElementsByTagName("script");
			const lastScriptTag = scripts[scripts.length - 1];

			if (lastScriptTag) {
				({ src } = lastScriptTag);
			}
		}

		srcByModuleId[moduleId] = src;
	}

	return (fileMap: string): Option<Array<string>> | null => {
		if (!src) {
			return null;
		}

		const splitResult = src.match(/([^\\/]+)\.js$/);
		// biome-ignore lint/complexity/useOptionalChain: not use optionalChain to support legacy browser
		const filename = splitResult && splitResult[1];

		if (!filename || !fileMap) {
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

function updateCss(el: HTMLLinkElement & Record<string, any>, url?: string) {
	let normalizedUrl;
	if (!url) {
		if (!el.href) {
			return;
		}

		normalizedUrl = el.href.split("?")[0];
	} else {
		normalizedUrl = url;
	}

	if (!isUrlRequest(normalizedUrl)) {
		return;
	}

	if (el.isLoaded === false) {
		// We seem to be about to replace a css link that hasn't loaded yet.
		// We're probably changing the same file more than once.
		return;
	}

	if (!normalizedUrl || !(normalizedUrl.indexOf(".css") > -1)) {
		return;
	}

	el.visited = true;

	const newEl = el.cloneNode() as Node & Record<string, any>;

	newEl.isLoaded = false;

	newEl.addEventListener("load", () => {
		if (newEl.isLoaded) {
			return;
		}

		newEl.isLoaded = true;
		if (el.parentNode) {
			el.parentNode.removeChild(el);
		}
	});

	newEl.addEventListener("error", () => {
		if (newEl.isLoaded) {
			return;
		}

		newEl.isLoaded = true;
		if (el.parentNode) {
			el.parentNode.removeChild(el);
		}
	});

	newEl.href = `${normalizedUrl}?${Date.now()}`;

	const parent = el.parentNode;

	if (!parent) {
		return;
	}

	if (el.nextSibling) {
		parent.insertBefore(newEl, el.nextSibling);
	} else {
		parent.appendChild(newEl);
	}
}

function getReloadUrl(href: string, src: Array<string>): string {
	let ret = "";

	const normalizedHref = normalizeUrl(href);

	src.some(url => {
		if (normalizedHref.indexOf(src as unknown as string) > -1) {
			ret = url;
		}
	});

	return ret;
}

function reloadStyle(src: Option<Array<string>>): boolean {
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

function isUrlRequest(url: string): boolean {
	// An URL is not an request if

	// It is not http or https
	if (!/^[a-zA-Z][a-zA-Z\d+\-.]*:/.test(url)) {
		return false;
	}

	return true;
}

function cssReload(moduleId: string, options: Record<string, any>) {
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
			// biome-ignore lint/complexity/useOptionalChain: not use optionalChain to support legacy browser
			console.log("[HMR] css reload %s", src && src.join(" "));
		} else {
			console.log("[HMR] Reload all css");

			reloadAll();
		}
	}

	return debounce(update, 50);
}

export { cssReload };
