/* eslint-env browser */
/* global __webpack_public_path__ */
/* eslint-disable no-console, camelcase, no-global-assign */

import "./initial.css";
import "./simple.css";
import classes from "./simple.module.css";

console.log("___CLASSES__");
console.log(classes);

function replaceClass(originalClass, newClass) {
	const nodes = document.querySelectorAll(`.${originalClass}`);

	nodes.forEach(node => {
		const { classList } = node;
		classList.remove(originalClass);
		classList.add(newClass);
	});
}

Object.keys(classes).forEach(localClass => {
	replaceClass(localClass, classes[localClass]);
});

let oldClasses = classes;

if (module.hot) {
	module.hot.accept("./simple.module.css", () => {
		Object.keys(oldClasses).forEach(localClass => {
			replaceClass(oldClasses[localClass], localClass);
		});
		Object.keys(classes).forEach(localClass => {
			replaceClass(localClass, classes[localClass]);
		});
		oldClasses = classes;
		// eslint-disable-next-line no-alert
		alert("HMR updated CSS module");
	});
}

const handleError = err => {
	document.querySelector(".errors").textContent += `\n${err.toString()}`;
	console.error(err);
};

const makeButton = (className, fn, shouldDisable = true) => {
	const button = document.querySelector(className);
	button.addEventListener("click", () => {
		if (shouldDisable) {
			button.disabled = true;
		}
		fn()
			.then(() => {
				button.disabled = false;
			})
			.catch(handleError);
	});
};

makeButton(".lazy-button", () => import("./lazy"));
makeButton(".lazy-button2", () => import("./lazy2.css"));
makeButton(".lazy-module-button", () =>
	import("./lazy.module.css").then(module => {
		console.log(module);
		document
			.querySelector(".lazy-css-module")
			// eslint-disable-next-line no-underscore-dangle
			.classList.add(module.__esModule ? module.default.style : module.style);
	})
);

makeButton(".preloaded-button1", () =>
	import(/* webpackChunkName: "preloaded1" */ "./preloaded1")
);
makeButton(".preloaded-button2", () =>
	import(/* webpackChunkName: "preloaded2" */ "./preloaded2")
);

makeButton(".lazy-failure-button", () => import("./lazy-failure"), false);

makeButton(".crossorigin", () => {
	const originalPublicPath = __webpack_public_path__;
	__webpack_public_path__ = "http://127.0.0.1:8080/dist/";
	const promise = import("./crossorigin").then(() => {
		const lastTwoElements = Array.from(document.head.children).slice(-2);
		const hasCrossorigin = lastTwoElements.every(
			element => element.crossOrigin === "anonymous"
		);
		if (!hasCrossorigin) {
			throw new Error('Chunks miss crossorigin="anonymous" attribute.');
		}
	});
	__webpack_public_path__ = originalPublicPath;
	return promise;
});

const worker = new Worker(new URL("./worker.js", import.meta.url));

worker.postMessage("test");

worker.addEventListener("message", event => {
	console.log(`Received message from worker: ${event.data}`);
});
