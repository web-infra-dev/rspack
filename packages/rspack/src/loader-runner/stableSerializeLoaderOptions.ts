export function stableSerializeLoaderOptions(
  value: unknown,
  path: string,
): string {
  const seen = new Set<object>();

  const serialize = (current: unknown, currentPath: string): string => {
    if (current === undefined) return 'undefined';
    if (current === null) return 'null';

    switch (typeof current) {
      case 'string':
        return `string:${JSON.stringify(current)}`;
      case 'boolean':
        return `boolean:${current}`;
      case 'number':
        return `number:${Object.is(current, -0) ? '-0' : String(current)}`;
      case 'bigint':
        return `bigint:${current.toString()}`;
      case 'function':
      case 'symbol':
        throw new Error(
          `\`Rule.use.cache\` requires stably serializable loader options. ` +
            `The value at \`${currentPath}\` has unsupported type \`${typeof current}\`.`,
        );
      case 'object':
        break;
      default:
        return `${typeof current}:${String(current)}`;
    }

    const object = current as object;
    if (seen.has(object)) {
      throw new Error(
        `\`Rule.use.cache\` requires stably serializable loader options. ` +
          `The value at \`${currentPath}\` contains a circular reference.`,
      );
    }
    seen.add(object);

    let result: string;
    if (Array.isArray(object)) {
      const extraKeys = Object.keys(object).filter(
        (key) => !/^\d+$/.test(key) || Number(key) >= object.length,
      );
      if (
        extraKeys.length > 0 ||
        Object.getOwnPropertySymbols(object).length > 0
      ) {
        throw new Error(
          `\`Rule.use.cache\` requires stably serializable loader options. ` +
            `The array at \`${currentPath}\` contains custom properties.`,
        );
      }
      result = `array:[${Array.from({ length: object.length }, (_, index) =>
        index in object
          ? serialize(object[index], `${currentPath}[${index}]`)
          : 'hole',
      ).join(',')}]`;
    } else if (object instanceof RegExp) {
      result = `regexp:${object.source}/${object.flags}/${object.lastIndex}`;
    } else if (object instanceof Date) {
      result = `date:${object.toISOString()}`;
    } else if (object instanceof URL) {
      result = `url:${object.toString()}`;
    } else if (object instanceof Map) {
      result = `map:{${[...object.entries()]
        .map(
          ([key, item], index) =>
            `${serialize(key, `${currentPath}.<key:${index}>`)}:${serialize(item, `${currentPath}.<value:${index}>`)}`,
        )
        .join(',')}}`;
    } else if (object instanceof Set) {
      result = `set:{${[...object.values()]
        .map((item, index) => serialize(item, `${currentPath}.<set:${index}>`))
        .join(',')}}`;
    } else {
      const prototype = Object.getPrototypeOf(object);
      if (prototype !== Object.prototype && prototype !== null) {
        throw new Error(
          `\`Rule.use.cache\` requires stably serializable loader options. ` +
            `The value at \`${currentPath}\` is an unsupported \`${object.constructor?.name ?? 'object'}\` instance.`,
        );
      }
      if (Object.getOwnPropertySymbols(object).length > 0) {
        throw new Error(
          `\`Rule.use.cache\` requires stably serializable loader options. ` +
            `The value at \`${currentPath}\` contains symbol keys.`,
        );
      }
      const keys = Object.keys(object).sort();
      if (Object.getOwnPropertyNames(object).length !== keys.length) {
        throw new Error(
          `\`Rule.use.cache\` requires stably serializable loader options. ` +
            `The value at \`${currentPath}\` contains non-enumerable properties.`,
        );
      }
      result = `object:{${keys
        .map((key) => {
          const descriptor = Object.getOwnPropertyDescriptor(object, key);
          if (descriptor?.get || descriptor?.set) {
            throw new Error(
              `\`Rule.use.cache\` requires stably serializable loader options. ` +
                `The value at \`${currentPath}.${key}\` is an accessor property.`,
            );
          }
          return `${JSON.stringify(key)}:${serialize(
            (object as Record<string, unknown>)[key],
            `${currentPath}.${key}`,
          )}`;
        })
        .join(',')}}`;
    }

    seen.delete(object);
    return result;
  };

  return serialize(value, path);
}
