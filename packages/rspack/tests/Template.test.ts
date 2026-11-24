import { describe, expect, it } from 'rstack/test';
import { Template } from '../src/Template';

describe('Template.getFunctionContent', () => {
  it('should strip named function wrappers', () => {
    const content = Template.getFunctionContent(
      function moduleFederationDefaultRuntime_default() {
        return 42;
      },
    );

    expect(content.trim()).toBe('return 42;');
  });

  it('should strip anonymous function wrappers', () => {
    const content = Template.getFunctionContent(function () {
      return 42;
    });

    expect(content.trim()).toBe('return 42;');
  });
});

describe('Template.getModulesArrayBounds', () => {
  it('should return false for empty arrays and negative or non-integer module ids', () => {
    expect(Template.getModulesArrayBounds([])).toBe(false);
    expect(Template.getModulesArrayBounds([{ id: -1 }])).toBe(false);
    expect(Template.getModulesArrayBounds([{ id: 1.5 }])).toBe(false);
    expect(Template.getModulesArrayBounds([{ id: Number.NaN }])).toBe(false);
  });

  it('should return bounds for dense non-negative integer module ids', () => {
    expect(
      Template.getModulesArrayBounds([{ id: 0 }, { id: 1 }, { id: 2 }]),
    ).toEqual([0, 2]);
  });
});
