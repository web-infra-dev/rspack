import type { RspackOptions } from '@rspack/core';
import { stringify } from 'javascript-stringify';

export default function stringifyConfig(
  config: RspackOptions,
  verbose = false,
) {
  return stringify(
    config,
    (value, indent, stringify) => {
      // improve plugin output
      if (value?.__pluginName) {
        const prefix = `/* config.${value.__pluginType}('${value.__pluginName}') */\n`;
        const constructorExpression = value.__pluginPath
          ? // The path is stringified to ensure special characters are escaped
            // (such as the backslashes in Windows-style paths).
            `(require(${stringify(value.__pluginPath)}))`
          : value.__pluginConstructorName;

        if (constructorExpression) {
          // get correct indentation for args by stringifying the args array and
          // discarding the square brackets.
          const args = stringify(value.__pluginArgs)?.slice(1, -1);
          return `${prefix}new ${constructorExpression}(${args})`;
        }
        return (
          prefix +
          stringify(
            value.__pluginArgs?.length ? { args: value.__pluginArgs } : {},
          )
        );
      }

      // improve rule/use output
      if (value?.__ruleNames) {
        const ruleTypes = value.__ruleTypes;
        const prefix = `/* config.module${value.__ruleNames
          .map(
            (r: string, index: number) =>
              `.${ruleTypes ? ruleTypes[index] : 'rule'}('${r}')`,
          )
          .join('')}${
          value.__useName ? `.use('${value.__useName}')` : ``
        } */\n`;
        return prefix + stringify(value);
      }

      if (value?.__expression) {
        return value.__expression;
      }

      // shorten long functions
      if (typeof value === 'function') {
        if (!verbose && value.toString().length > 100) {
          return `function () { /* omitted long function */ }`;
        }
      }

      return stringify(value);
    },
    2,
  );
}
