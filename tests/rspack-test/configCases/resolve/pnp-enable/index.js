import { statSync } from 'node:fs';
import path from 'node:path';
import answer from 'the-answer';

it('load_dep_from_pnp', () => {
    expect(answer).toBe(42);
});

it('preserves the archived file timestamp', () => {
    const stat = statSync(path.join(__dirname, 'copied-package.json'));
    const archivedTime = Date.parse('1984-06-22T21:50:00Z');
    expect(stat.mtimeMs).toBe(archivedTime);
    expect(stat.atimeMs).toBe(archivedTime);
});

it('preserves the archived file permissions', () => {
    const stat = statSync(path.join(__dirname, 'copied-executable.txt'));
    if (process.platform !== 'win32') {
        expect(stat.mode & 0o777).toBe(0o751);
    }
});
