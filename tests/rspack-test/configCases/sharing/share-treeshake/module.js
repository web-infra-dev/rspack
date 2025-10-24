import react from "react";

export const value = react;

export async function load() {
	return import("./lazy-module");
}
