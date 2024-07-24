import { test as base, expect } from "@playwright/test";
import { pathInfoFixtures } from "./pathInfo";
import { rspackFixtures, type RspackOptions } from "./rspack";
import { fileActionFixtures } from "./fileAction";

const test = base
	.extend(pathInfoFixtures)
	.extend(rspackFixtures(true))
	.extend(rspackFixtures(false))
	.extend(fileActionFixtures);

export type { RspackOptions };
export { test, expect };
