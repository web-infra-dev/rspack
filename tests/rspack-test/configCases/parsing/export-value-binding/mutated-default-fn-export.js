export default function fn() {
	return "default-fn";
}

export function setFn(value) {
	fn = value;
}
