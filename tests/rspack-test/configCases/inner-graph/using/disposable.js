export const log = [];

export const resource = {
	[Symbol.dispose]() {
		log.push("resource");
	}
};

export function createResource(name) {
	return {
		[Symbol.dispose]() {
			log.push(name);
		}
	};
}
