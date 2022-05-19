import chai from "chai";
import { jestSnapshotPlugin } from "mocha-chai-jest-snapshot";

import { SourceMapConsumer } from "source-map"
import convertSourceMap from "convert-source-map"

chai.use(jestSnapshotPlugin());

const expect = chai.expect;

export { expect, SourceMapConsumer, convertSourceMap };