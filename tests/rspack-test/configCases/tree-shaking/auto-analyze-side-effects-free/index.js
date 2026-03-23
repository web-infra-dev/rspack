import { pureEmpty } from './pure-empty';
import { pureReturnLiteral } from './pure-return';
import { pureArrow } from './pure-arrow';
import { pureLocalConst } from './pure-local-const';
import { pureUsesHelper } from './pure-local-call';
import { pureStatements } from './pure-statements';
import { impureDefaultParam } from './impure-default-param';
import { impureBodyCall } from './impure-body-call';
import { impureGlobalWrite } from './impure-global-write';
import { impureObjectPattern } from './impure-object-pattern';
import { impureMemberAccess } from './impure-member-access';
import { impureLocalAssignment } from './impure-local-assignment';
import { impureIf } from './impure-if';
import { callImportedPure } from './impure-imported-call';
import { reassignedFunction } from './reassigned-function';
import { events } from './tracker';

const unusedPureEmpty = pureEmpty();
const unusedPureReturnLiteral = pureReturnLiteral();
const unusedPureArrow = pureArrow();
const unusedPureLocalConst = pureLocalConst();
const unusedPureUsesHelper = pureUsesHelper();
const unusedPureStatements = pureStatements();

impureDefaultParam();
impureBodyCall();
impureGlobalWrite();

let objectPatternGetterCount = 0;
impureObjectPattern({
  get value() {
    objectPatternGetterCount += 1;
    return 1;
  }
});

let memberAccessGetterCount = 0;
impureMemberAccess({
  get value() {
    memberAccessGetterCount += 1;
    return 1;
  }
});

impureLocalAssignment();
impureIf(false);
callImportedPure();
reassignedFunction();

it('should auto analyze no-side-effects functions conservatively', () => {
  expect(events).toEqual([
    'default-param',
    'body-call',
    'global-write',
    'reassigned'
  ]);
  expect(globalThis.__AUTO_SIDE_EFFECTS_WRITE__).toBe(1);
  expect(objectPatternGetterCount).toBe(1);
  expect(memberAccessGetterCount).toBe(1);
  expect(Reflect.ownKeys(__webpack_modules__).length).toBe(12);
});
