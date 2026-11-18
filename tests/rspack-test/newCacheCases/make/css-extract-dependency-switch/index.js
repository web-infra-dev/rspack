import './style.css';

it('should update extracted CSS when a dependency changes after cache restoration', async () => {
  if (COMPILER_INDEX === 0) {
    await NEXT_START();
  } else {
    await NEXT_HMR();
  }
});
