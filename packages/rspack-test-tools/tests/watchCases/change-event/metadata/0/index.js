const a = 1;

it('should compile', () => {
  switch (WATCH_STEP) {
    case "0":
      // do nothing
      break;
    case "1":
      expect(GLOBAL_WATCH_CHANGE_COUNT).toBe(1)

      break;
    default:
      throw new Error('unexpected update');
  }
})
