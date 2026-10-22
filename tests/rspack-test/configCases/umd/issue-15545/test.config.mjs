const CONTEXT = {};
export default {
	nonEsmThis(module) {
		return CONTEXT;
	},
	findBundle() {
		return ["./runtime.js", "./main.js"];
	}
};
