module.exports = {
	documentType: "fake",
	moduleScope(scope) {
		const link1 = scope.window.document.createElement("link");
		link1.rel = "stylesheet";
		link1.href = "bundle0.css";
		scope.window.document.head.appendChild(link1);
	}
};
