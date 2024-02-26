/**
 @jest-environment jsdom
 */
/* eslint-env browser */
/* eslint-disable no-console */

import hotModuleReplacement from "../../src/builtin-plugin/mini-css-extract/hmr/hotModuleReplacement";

function getLoadEvent() {
	const event = document.createEvent("Event");

	event.initEvent("load", false, false);

	return event;
}

function getErrorEvent() {
	const event = document.createEvent("Event");

	event.initEvent("error", false, false);

	return event;
}

describe("HMR", () => {
	let consoleMock = null;

	beforeEach(() => {
		consoleMock = jest.spyOn(console, "log").mockImplementation(() => () => {});

		jest.spyOn(Date, "now").mockImplementation(() => 1479427200000);

		document.head.innerHTML = '<link rel="stylesheet" href="/dist/main.css" />';
		document.body.innerHTML = '<script src="/dist/main.js"></script>';
	});

	afterEach(() => {
		consoleMock.mockClear();
	});

	it("should works", done => {
		const update = hotModuleReplacement("./src/style.css", {});

		update();

		setTimeout(() => {
			expect(console.log.mock.calls[0][0]).toMatchSnapshot();

			const links = Array.prototype.slice.call(
				document.querySelectorAll("link")
			);

			expect(links[0].visited).toBe(true);
			expect(document.head.innerHTML).toMatchSnapshot();

			links[1].dispatchEvent(getLoadEvent());

			expect(links[1].isLoaded).toBe(true);

			done();
		}, 100);
	});

	it("should works with multiple updates", done => {
		const update = hotModuleReplacement("./src/style.css", {});

		update();

		setTimeout(() => {
			expect(console.log.mock.calls[0][0]).toMatchSnapshot();

			const links = Array.prototype.slice.call(
				document.querySelectorAll("link")
			);

			expect(links[0].visited).toBe(true);
			expect(document.head.innerHTML).toMatchSnapshot();

			links[1].dispatchEvent(getLoadEvent());

			expect(links[1].isLoaded).toBe(true);

			jest.spyOn(Date, "now").mockImplementation(() => 1479427200001);

			const update2 = hotModuleReplacement("./src/style.css", {});

			update2();

			setTimeout(() => {
				const links2 = Array.prototype.slice.call(
					document.querySelectorAll("link")
				);

				expect(links2[0].visited).toBe(true);
				expect(links2[0].isLoaded).toBe(true);
				expect(document.head.innerHTML).toMatchSnapshot();

				links2[1].dispatchEvent(getLoadEvent());

				expect(links2[1].isLoaded).toBe(true);

				done();
			}, 100);
		}, 100);
	});

	it("should reloads with locals", done => {
		const update = hotModuleReplacement("./src/style.css", {
			locals: { foo: "bar" }
		});

		update();

		setTimeout(() => {
			expect(console.log.mock.calls[0][0]).toMatchSnapshot();

			const links = Array.prototype.slice.call(
				document.querySelectorAll("link")
			);

			expect(links[0].visited).toBe(true);
			expect(document.head.innerHTML).toMatchSnapshot();

			links[1].dispatchEvent(getLoadEvent());

			expect(links[1].isLoaded).toBe(true);

			done();
		}, 100);
	});

	it("should work reload all css", done => {
		const update = hotModuleReplacement("./src/style.css", {
			filename: "unreload_url"
		});

		update();

		setTimeout(() => {
			expect(console.log.mock.calls[0][0]).toMatchSnapshot();

			const links = Array.prototype.slice.call(
				document.querySelectorAll("link")
			);

			expect(links[0].visited).toBe(true);
			expect(document.head.innerHTML).toMatchSnapshot();

			links[1].dispatchEvent(getLoadEvent());

			expect(links[1].isLoaded).toBe(true);

			done();
		}, 100);
	});

	it("should reloads with non http/https link href", done => {
		document.head.innerHTML =
			'<link rel="stylesheet" href="/dist/main.css" /><link rel="shortcut icon" href="data:;base64,=" />';

		const update = hotModuleReplacement("./src/style.css", {});

		update();

		setTimeout(() => {
			expect(console.log.mock.calls[0][0]).toMatchSnapshot();

			const links = Array.prototype.slice.call(
				document.querySelectorAll("link")
			);

			expect(links[0].visited).toBe(true);
			expect(document.head.innerHTML).toMatchSnapshot();

			links[1].dispatchEvent(getLoadEvent());

			expect(links[1].isLoaded).toBe(true);
			expect(links[2].visited).toBeUndefined();

			done();
		}, 100);
	});

	it("should reloads with # link href", done => {
		document.head.innerHTML =
			'<link rel="stylesheet" href="/dist/main.css" /><link rel="shortcut icon" href="#href" />';

		const update = hotModuleReplacement("./src/style.css", {});

		update();

		setTimeout(() => {
			expect(console.log.mock.calls[0][0]).toMatchSnapshot();

			const links = Array.prototype.slice.call(
				document.querySelectorAll("link")
			);

			expect(links[0].visited).toBe(true);
			expect(document.head.innerHTML).toMatchSnapshot();

			links[1].dispatchEvent(getLoadEvent());

			expect(links[1].isLoaded).toBe(true);
			expect(links[2].visited).toBeUndefined();

			done();
		}, 100);
	});

	it("should reloads with link without href", done => {
		document.head.innerHTML =
			'<link rel="stylesheet" href="/dist/main.css" /><link rel="shortcut icon" />';

		const update = hotModuleReplacement("./src/style.css", {});

		update();

		setTimeout(() => {
			expect(console.log.mock.calls[0][0]).toMatchSnapshot();

			const links = Array.prototype.slice.call(
				document.querySelectorAll("link")
			);

			expect(links[0].visited).toBe(true);
			expect(document.head.innerHTML).toMatchSnapshot();

			links[1].dispatchEvent(getLoadEvent());

			expect(links[1].isLoaded).toBe(true);
			expect(links[2].visited).toBeUndefined();

			done();
		}, 100);
	});

	it("should reloads with absolute remove url", done => {
		document.head.innerHTML =
			'<link rel="stylesheet" href="/dist/main.css" /><link rel="stylesheet" href="http://dev.com/dist/main.css" />';

		const update = hotModuleReplacement("./src/style.css", {});

		update();

		setTimeout(() => {
			expect(console.log.mock.calls[0][0]).toMatchSnapshot();

			const links = Array.prototype.slice.call(
				document.querySelectorAll("link")
			);

			expect(links[0].visited).toBe(true);
			expect(document.head.innerHTML).toMatchSnapshot();

			links[1].dispatchEvent(getLoadEvent());

			expect(links[1].isLoaded).toBe(true);
			expect(links[2].visited).toBeUndefined();

			done();
		}, 100);
	});

	it("should reloads with browser extension protocol", done => {
		document.head.innerHTML =
			'<link rel="stylesheet" href="/dist/main.css" /><link rel="stylesheet" href="chrome-extension://main.css" />';

		const update = hotModuleReplacement("./src/style.css", {});

		update();

		setTimeout(() => {
			expect(console.log.mock.calls[0][0]).toMatchSnapshot();

			const links = Array.prototype.slice.call(
				document.querySelectorAll("link")
			);

			expect(links[0].visited).toBe(true);
			expect(document.head.innerHTML).toMatchSnapshot();

			links[1].dispatchEvent(getLoadEvent());

			expect(links[1].isLoaded).toBe(true);
			expect(links[2].visited).toBeUndefined();

			done();
		}, 100);
	});

	it("should reloads with non-file script in the end of page", done => {
		document.body.appendChild(document.createElement("script"));

		const update = hotModuleReplacement("./src/non_file_styles.css", {});

		update();

		setTimeout(() => {
			expect(console.log.mock.calls[0][0]).toMatchSnapshot();

			const links = Array.prototype.slice.call(
				document.querySelectorAll("link")
			);

			expect(links[0].visited).toBe(true);
			expect(document.head.innerHTML).toMatchSnapshot();

			links[1].dispatchEvent(getLoadEvent());

			expect(links[1].isLoaded).toBe(true);

			done();
		}, 100);
	});

	it("should handle error event", done => {
		const update = hotModuleReplacement("./src/style.css", {});

		update();

		setTimeout(() => {
			expect(console.log.mock.calls[0][0]).toMatchSnapshot();

			const links = Array.prototype.slice.call(
				document.querySelectorAll("link")
			);

			expect(links[0].visited).toBe(true);
			expect(document.head.innerHTML).toMatchSnapshot();

			links[1].dispatchEvent(getErrorEvent());

			expect(links[1].isLoaded).toBe(true);

			done();
		}, 100);
	});

	it("should not remove old link when new link is loaded twice", done => {
		const link = document.createElement("link");

		link.innerHTML = '<link rel="preload stylesheet" href="./dist/main.css" />';
		document.head.appendChild(link);
		document.head.removeChild = jest.fn();

		const update = hotModuleReplacement("./dist/main.css", {});

		update();

		setTimeout(() => {
			const links = Array.prototype.slice.call(
				document.querySelectorAll("link")
			);

			links[1].dispatchEvent(getLoadEvent());
			links[1].dispatchEvent(getLoadEvent());

			expect(document.head.removeChild).toHaveBeenCalledTimes(1);

			done();
		}, 100);
	});
});
