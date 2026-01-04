export const memoize = <T>(fn: () => T): (() => T) => {
  let cache = false;
  let result: T;
  let callback = fn;

  return () => {
    if (cache) {
      return result;
    }
    result = callback();
    cache = true;
    // Allow to clean up memory for fn
    // and all dependent resources
    callback = undefined!;
    return result;
  };
};

// Lazily init a function, and cache it afterwards.
export const memoizeFn = <const T extends readonly unknown[], const P>(
  fn: () => (...args: T) => P,
) => {
  let cache: ((...args: T) => P) | null = null;
  return (...args: T) => {
    if (!cache) {
      cache = fn();
    }
    return cache(...args);
  };
};
