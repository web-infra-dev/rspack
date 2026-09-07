import './trigger.js';
import targetModuleId from './module7.js';

it('should invalidate cached code generation when global module ids change', () => {
  const targetModule = __STATS__.modules.find(
    (module) => module.name === './module7.js',
  );
  expect(targetModule).toBeTruthy();

  if (WATCH_STEP === '0') {
    STATE.targetModuleId = targetModule.id;
  } else {
    expect(targetModule.id).not.toBe(STATE.targetModuleId);
  }

  expect(targetModuleId).toBe(targetModule.id);
});
