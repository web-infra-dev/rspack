import cycleDefault from "./cycle-middle";

export let observed;
try {
	observed = cycleDefault;
} catch (error) {
	observed = `${error.name}:${error.message}`;
}

export const value = 1;
