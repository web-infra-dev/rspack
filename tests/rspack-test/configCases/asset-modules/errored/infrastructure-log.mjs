export default (options) => {
	if (options.cache && options.cache.type === "persistent") {
		return [/Pack got invalid because of write to/];
	}

	return [];
};
