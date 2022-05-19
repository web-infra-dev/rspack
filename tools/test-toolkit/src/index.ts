import chai from "chai";
import { jestSnapshotPlugin } from "mocha-chai-jest-snapshot";

chai.use(jestSnapshotPlugin());

const expect = chai.expect;

export { expect }