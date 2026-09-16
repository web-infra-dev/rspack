import {
  expect as playwrightExpect,
  type LocatorAssertions,
  type MatcherOptions,
  type PageAssertions,
} from '@rstest/playwright';
import { rstest } from 'rstack/test';

// @rstest/playwright has no global matcher timeout yet. Keep browser assertions
// aligned with the suite's testTimeout, while allowing per-assertion overrides.
const optionsIndex = {
  toBeVisible: 0,
  toBeHidden: 0,
  toBeEnabled: 0,
  toBeDisabled: 0,
  toBeChecked: 0,
  toBeUnchecked: 0,
  toBeAttached: 0,
  toBeDetached: 0,
  toBeEditable: 0,
  toBeFocused: 0,
  toBeEmpty: 0,
  toBeInViewport: 0,
  toContainText: 1,
  toHaveAttribute: 2,
  toHaveClass: 1,
  toHaveCSS: 2,
  toHaveCount: 1,
  toHaveId: 1,
  toHaveJSProperty: 2,
  toHaveText: 1,
  toHaveValue: 1,
  toHaveTitle: 1,
  toHaveURL: 1,
} satisfies Record<
  Exclude<keyof LocatorAssertions | keyof PageAssertions, 'not'>,
  number
>;

function withTimeout(assertion: object): object {
  return new Proxy(assertion, {
    get(target, key, receiver) {
      const value = Reflect.get(target, key, receiver);
      if (key === 'not') return withTimeout(value);
      if (!Object.prototype.hasOwnProperty.call(optionsIndex, key))
        return value;

      return (...args: unknown[]) => {
        const index = optionsIndex[key as keyof typeof optionsIndex];
        const options = args[index] as MatcherOptions | undefined;
        args[index] = {
          ...options,
          timeout: options?.timeout ?? rstest.getConfig().testTimeout,
        };
        return Reflect.apply(value, target, args);
      };
    },
  });
}

function withDefaultTimeout(
  base: typeof playwrightExpect,
): typeof playwrightExpect {
  return new Proxy(base, {
    apply(target, thisArg, args) {
      return withTimeout(Reflect.apply(target, thisArg, args));
    },
    get(target, key, receiver) {
      const value = Reflect.get(target, key, receiver);
      return key === 'soft' ? withDefaultTimeout(value) : value;
    },
  });
}

export const expect = withDefaultTimeout(playwrightExpect);
