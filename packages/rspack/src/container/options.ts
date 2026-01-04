type ContainerOptionsFormat<T> =
  | (string | Record<string, string | string[] | T>)[]
  | Record<string, string | string[] | T>;

const process = <T, N>(
  options: ContainerOptionsFormat<T>,
  normalizeSimple: (a: string | string[], b: string) => N,
  normalizeOptions: (a: T, b: string) => N,
  fn: (a: string, b: N) => void,
) => {
  const array = (items: (string | Record<string, string | string[] | T>)[]) => {
    for (const item of items) {
      if (typeof item === 'string') {
        fn(item, normalizeSimple(item, item));
      } else if (item && typeof item === 'object') {
        object(item);
      } else {
        throw new Error('Unexpected options format');
      }
    }
  };
  const object = (obj: Record<string, string | string[] | T>) => {
    for (const [key, value] of Object.entries(obj)) {
      if (typeof value === 'string' || Array.isArray(value)) {
        fn(key, normalizeSimple(value, key));
      } else {
        fn(key, normalizeOptions(value, key));
      }
    }
  };
  if (!options) {
    return;
  }
  if (Array.isArray(options)) {
    array(options);
  } else if (typeof options === 'object') {
    object(options);
  } else {
    throw new Error('Unexpected options format');
  }
};

export const parseOptions = <T, R>(
  options: ContainerOptionsFormat<T>,
  normalizeSimple: (a: string | string[], b: string) => R,
  normalizeOptions: (a: T, b: string) => R,
) => {
  const items: [string, R][] = [];
  process(options, normalizeSimple, normalizeOptions, (key, value) => {
    items.push([key, value]);
  });
  return items;
};
