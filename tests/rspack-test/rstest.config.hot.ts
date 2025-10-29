import { defineConfig } from '@rstest/core';
import config from './rstest.config'

export default defineConfig({
	...config,
	include: process.env.WASM ? [] :["<rootDir>/*.hottest.js"]
});
