export const events = (globalThis.__modern_module_defer_events__ = []);

export function reset() {
	events.length = 0;
}
