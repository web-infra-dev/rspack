module.exports = {
	moduleScope(scope) {
		const link1 = scope.window.document.createElement("link");
		link1.rel = "stylesheet";
		link1.href = "imported_js.bundle0.css";
		scope.window.document.head.appendChild(link1);

		const link2 = scope.window.document.createElement("link");
		link2.rel = "stylesheet";
		link2.href = "reexported_js.bundle0.css";
		scope.window.document.head.appendChild(link2);

		const link3 = scope.window.document.createElement("link");
		link3.rel = "stylesheet";
		link3.href = "style_module_css.bundle0.css";
		scope.window.document.head.appendChild(link3);
	}
};
