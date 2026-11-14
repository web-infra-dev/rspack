import fs from 'node:fs';
import { execFileSync } from 'node:child_process';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';

const require = createRequire(import.meta.url);
const { NativeWatcher } = createRequire(require.resolve('@rspack/core'))(
  '@rspack/binding',
);

// libuv creates an empty directory before setting its junction reparse point.
const setJunction = String.raw`
Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.Text;
using System.ComponentModel;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;
public static class Junction {
  [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
  static extern SafeFileHandle CreateFile(string name, uint access, uint share, IntPtr security, uint creation, uint flags, IntPtr template);
  [DllImport("kernel32.dll", SetLastError = true)]
  static extern bool DeviceIoControl(SafeFileHandle handle, uint code, byte[] input, int size, IntPtr output, int outputSize, out int returned, IntPtr overlapped);
  public static void SetTarget(string directory, string target) {
    byte[] substitute = Encoding.Unicode.GetBytes(@"\??\" + target);
    byte[] print = Encoding.Unicode.GetBytes(target);
    using (var stream = new MemoryStream()) {
      using (var writer = new BinaryWriter(stream)) {
        writer.Write(0xA0000003u);
        writer.Write((ushort)(12 + substitute.Length + print.Length));
        writer.Write((ushort)0);
        writer.Write((ushort)0);
        writer.Write((ushort)substitute.Length);
        writer.Write((ushort)(substitute.Length + 2));
        writer.Write((ushort)print.Length);
        writer.Write(substitute);
        writer.Write((ushort)0);
        writer.Write(print);
        writer.Write((ushort)0);
        byte[] buffer = stream.ToArray();
        using (var handle = CreateFile(directory, 0x40000000u, 0, IntPtr.Zero, 3, 0x02200000u, IntPtr.Zero)) {
          if (handle.IsInvalid) throw new Win32Exception(Marshal.GetLastWin32Error());
          int returned;
          if (!DeviceIoControl(handle, 0x000900A4u, buffer, buffer.Length, IntPtr.Zero, 0, out returned, IntPtr.Zero))
            throw new Win32Exception(Marshal.GetLastWin32Error());
        }
      }
    }
  }
}
'@
[Junction]::SetTarget($env:RSPACK_TEST_JUNCTION, $env:RSPACK_TEST_TARGET)
`;

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
export default process.platform === 'win32'
  ? [
      {
        description: 'should scan a directory when it becomes a junction',
        async run() {
          const root = fs.mkdtempSync(
            path.join(
              fs.realpathSync.native(os.tmpdir()),
              'rspack-junction-transition-',
            ),
          );
          const context = path.join(root, 'app');
          const target = path.join(root, 'outside');
          const link = path.join(context, 'link');
          fs.mkdirSync(context);
          fs.mkdirSync(target);
          fs.writeFileSync(path.join(target, 'value.js'), 'initial');
          const watcher = new NativeWatcher({
            followSymlinks: true,
            aggregateTimeout: 50,
          });
          const changes = new Set();
          let error;
          const waitFor = async (predicate) => {
            const deadline = Date.now() + 5000;
            while (!predicate() && Date.now() < deadline) await sleep(10);
            expect(predicate()).toBe(true);
          };
          try {
            watcher.watch(
              [[], []],
              [[context], []],
              [[], []],
              BigInt(Date.now()),
              (err, batch) => {
                if (err) error = err;
                else for (const file of batch.changedFiles) changes.add(file);
              },
              () => {},
            );
            await sleep(500);
            changes.clear();
            fs.mkdirSync(link);
            await waitFor(() => changes.has(context));
            expect(
              watcher
                .collectTimeInfoEntries()
                .directoryTimestamps.entries.some(
                  (entry) => entry.path === link,
                ),
            ).toBe(true);
            execFileSync(
              'powershell.exe',
              ['-NoProfile', '-NonInteractive', '-Command', setJunction],
              {
                env: {
                  ...process.env,
                  RSPACK_TEST_JUNCTION: link,
                  RSPACK_TEST_TARGET: target,
                },
              },
            );
            expect(fs.lstatSync(link).isSymbolicLink()).toBe(true);
            const alias = path.join(link, 'value.js');
            await waitFor(() =>
              watcher
                .collectTimeInfoEntries()
                .fileTimestamps.entries.some((entry) => entry.path === alias),
            );
            expect(error).toBeUndefined();
          } finally {
            await watcher.close();
            fs.rmSync(root, { recursive: true, force: true });
          }
        },
      },
    ]
  : [];
