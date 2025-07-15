import { test as base, expect } from "@playwright/test";
import { fileActionFixtures } from "./fileAction";
import { pathInfoFixtures } from "./pathInfo";
import { type RspackOptions, rspackFixtures } from "./rspack";

const test = base
	.extend(pathInfoFixtures)
	.extend(rspackFixtures())
	.extend(fileActionFixtures);

export type { RspackOptions };
export { test, expect };
