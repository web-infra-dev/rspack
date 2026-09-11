import { test as base, expect } from './base';
import { pathInfoFixtures } from './pathInfo';
import { rspackFixtures } from './rspack';
import { fileActionFixtures } from './fileAction';

const test = base
  .extend(pathInfoFixtures)
  .extend(rspackFixtures)
  .extend(fileActionFixtures);

export { test, expect };
