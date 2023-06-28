import { test as base, expect } from "@playwright/test";
import { pathInfoFixtures } from "./pathInfo";
import { rspackFixtures, RspackOptions } from "./rspack";
import { fileActionFixtures } from "./fileAction";

const test = base
	.extend(pathInfoFixtures)
	.extend(rspackFixtures)
	.extend(fileActionFixtures);

export type { RspackOptions };
export { test, expect };
