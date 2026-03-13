"use strict";

module.exports = {
	validate(stats) {
		expect(stats.toJson({ modules: true }).children[0].modules.length).toBe(241);
	}
};
