export default {
	findBundle(index, options, step) {
		return step === "2" ? `bundle${index}.js` : [];
	}
};
