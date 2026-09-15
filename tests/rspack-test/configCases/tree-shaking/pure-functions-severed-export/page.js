import { mergeStore } from "./store";

export function setup() {
	return mergeStore({});
}

export default function render() {
	return "page";
}
