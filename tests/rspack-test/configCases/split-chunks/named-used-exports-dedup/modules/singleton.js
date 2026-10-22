let initialized = false;

export function init() {
  initialized = true;
}

export function track() {
  return initialized;
}
