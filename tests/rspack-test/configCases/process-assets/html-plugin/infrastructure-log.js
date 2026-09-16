module.exports = [
	// each time returns different OriginalSource in rspack.config.js
	// this prevents hit in inmemory cache
	/^Pack got invalid because of write to: RealContentHashPlugin|analyse|index\.html$/
];
