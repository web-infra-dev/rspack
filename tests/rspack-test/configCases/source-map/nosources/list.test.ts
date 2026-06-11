// Emulate the rstest e2e/list fixture layout.

describe('test a', () => {
  it('test a-1', () => {
    void 0;
  });
});

it('test a-2', () => {
  void 0;
});

function describe(_name: string, fn: () => void) {
  fn();
}

function it(_name: string, fn: () => void) {
  fn();
}
