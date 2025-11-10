"use strict";

module.exports = {
	moduleScope(scope, stats) {
		const __STATS_I__ = stats().__index__;
		const link = scope.window.document.createElement("link");
		link.rel = "stylesheet";
		link.href = `bundle${__STATS_I__}.css`;
		scope.window.document.head.appendChild(link);
	}
};
