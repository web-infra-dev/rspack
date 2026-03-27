export const log = [];

export function track(value) {
	log.push(value);
	return 0;
}
