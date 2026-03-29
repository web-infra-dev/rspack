import type { RuleSetRule } from '../src/config';

const validLoaderRule: RuleSetRule = {
  loader: 'builtin:swc-loader',
  options: {},
};

const validUseRule: RuleSetRule = {
  use: [
    {
      loader: 'builtin:swc-loader',
      options: {},
    },
  ],
};

// @ts-expect-error top-level options only apply to the loader shorthand
const invalidUseWithOptionsRule: RuleSetRule = {
  use: 'builtin:swc-loader',
  options: {},
};

// @ts-expect-error top-level options require the loader shorthand
const invalidOptionsWithoutLoaderRule: RuleSetRule = {
  options: {},
};

// @ts-expect-error loader and use shorthands are mutually exclusive
const invalidLoaderAndUseRule: RuleSetRule = {
  loader: 'builtin:swc-loader',
  use: 'builtin:swc-loader',
};

export const validRules = [validLoaderRule, validUseRule];

void invalidUseWithOptionsRule;
void invalidOptionsWithoutLoaderRule;
void invalidLoaderAndUseRule;
