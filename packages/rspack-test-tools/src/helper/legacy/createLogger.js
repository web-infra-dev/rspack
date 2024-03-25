// @ts-nocheck
module.exports = appendTarget => {
	return {
		log: l => appendTarget.push(l),
		debug: l => appendTarget.push(l),
		trace: l => appendTarget.push(l),
		info: l => appendTarget.push(l),
		warn: console.warn.bind(console),
		error: console.error.bind(console),
		logTime: () => {},
		group: () => {},
		groupCollapsed: () => {},
		groupEnd: () => {},
		profile: () => {},
		profileEnd: () => {},
		clear: () => {},
		status: () => {}
	};
};
