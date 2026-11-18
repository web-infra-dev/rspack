export default {
	findBundle() {
		// Only the page entry starts; the other entry never loads.
		return ['shared.js', 'page.js'];
	},
};
