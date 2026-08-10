module.exports = {
	get value() {
		process.__modern_module_defer_external_events__.push("external");
		return 9;
	},
};
