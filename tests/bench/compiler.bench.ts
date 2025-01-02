import { Rspack } from "@rspack/binding";
import { rspack } from "@rspack/core";
import { bench, describe, beforeAll, afterAll } from "vitest";
import rspackConfig from "./fixtures/ts-react/rspack.config";

const interruptMessage = 'This error is intentionally thrown to interrupt the subsequent process of Rspack';

let originRspackBuild;

beforeAll(() => {
    originRspackBuild = Rspack.prototype.build;
    Rspack.prototype.build = () => {
        throw new Error(interruptMessage);
    };
});

afterAll(() => {
    Rspack.prototype.build = originRspackBuild;
});

describe("Rspack compiler benchmark", () => {
	bench("Initialize compiler instance", async () => {
        await new Promise((resolve, reject) =>
            rspack(
                {
                    ...rspackConfig,
                    mode: "production"
                },
                (err, stats) => {
                    if (err?.message.includes(interruptMessage)) {
                        resolve(undefined);
                        return;
                    }
                    if (err) {
                        reject(err);
                        return;
                    }
                    if (stats?.hasErrors()) {
                        reject(new Error(stats.toString({})));
                    }
                }
            )
        );
	});
});
