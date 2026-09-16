import { define } from 'rstack';
import { e2eConfig } from '../e2e/config.ts';
import { appConfig } from './app.config.ts';

define.app(appConfig);

define.test({
  ...e2eConfig(120_000, 2),
  globalSetup: ['./globalSetup.ts'],
});
