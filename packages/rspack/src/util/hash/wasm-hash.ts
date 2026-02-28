/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/util/hash/wasm-hash.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

// 65536 is the size of a wasm memory page
// 64 is the maximum chunk size for every possible wasm hash implementation
// 4 is the maximum number of bytes per char for string encoding (max is utf-8)
// ~3 makes sure that it's always a block of 4 chars, so avoid partially encoded bytes for base64
const MAX_SHORT_STRING = Math.floor((65536 - 64) / 4) & ~3;

type Exports = WebAssembly.Instance['exports'] & {
  init: () => void;
  update: (b: number) => void;
  memory: WebAssembly.Memory;
  final: (b: number) => void;
};

export class WasmHash {
  exports: Exports;
  instancesPool: WebAssembly.Instance[];
  buffered: number;
  mem: Buffer;
  chunkSize: number;
  digestSize: number;

  /**
   * @param instance wasm instance
   * @param instancesPool pool of instances
   * @param chunkSize size of data chunks passed to wasm
   * @param digestSize size of digest returned by wasm
   */
  constructor(
    instance: WebAssembly.Instance,
    instancesPool: WebAssembly.Instance[],
    chunkSize: number,
    digestSize: number,
  ) {
    const exports = instance.exports as Exports;
    exports.init();

    this.exports = exports;
    this.mem = Buffer.from(exports.memory.buffer, 0, 65536);
    this.buffered = 0;
    this.instancesPool = instancesPool;
    this.chunkSize = chunkSize;
    this.digestSize = digestSize;
  }

  reset() {
    this.buffered = 0;
    this.exports.init();
  }

  /**
   * @param data data
   * @param encoding encoding
   * @returns itself
   */
  update(data: Buffer | string, encoding?: BufferEncoding): this {
    if (typeof data === 'string') {
      let normalizedData = data;
      while (normalizedData.length > MAX_SHORT_STRING) {
        this._updateWithShortString(
          normalizedData.slice(0, MAX_SHORT_STRING),
          encoding,
        );
        normalizedData = normalizedData.slice(MAX_SHORT_STRING);
      }
      this._updateWithShortString(normalizedData, encoding);
      return this;
    }
    this._updateWithBuffer(data);
    return this;
  }

  /**
   * @param {string} data data
   * @param {BufferEncoding=} encoding encoding
   * @returns {void}
   */
  _updateWithShortString(data: string, encoding?: BufferEncoding): void {
    const { exports, buffered, mem, chunkSize } = this;
    let endPos: number;
    if (data.length < 70) {
      if (!encoding || encoding === 'utf-8' || encoding === 'utf8') {
        endPos = buffered;
        for (let i = 0; i < data.length; i++) {
          const cc = data.charCodeAt(i);
          if (cc < 0x80) mem[endPos++] = cc;
          else if (cc < 0x800) {
            mem[endPos] = (cc >> 6) | 0xc0;
            mem[endPos + 1] = (cc & 0x3f) | 0x80;
            endPos += 2;
          } else {
            // bail-out for weird chars
            endPos += mem.write(data.slice(i), endPos, encoding);
            break;
          }
        }
      } else if (encoding === 'latin1') {
        endPos = buffered;
        for (let i = 0; i < data.length; i++) {
          const cc = data.charCodeAt(i);
          mem[endPos++] = cc;
        }
      } else {
        endPos = buffered + mem.write(data, buffered, encoding);
      }
    } else {
      endPos = buffered + mem.write(data, buffered, encoding);
    }
    if (endPos < chunkSize) {
      this.buffered = endPos;
    } else {
      const l = endPos & ~(this.chunkSize - 1);
      exports.update(l);
      const newBuffered = endPos - l;
      this.buffered = newBuffered;
      if (newBuffered > 0) mem.copyWithin(0, l, endPos);
    }
  }

  /**
   * @param data data
   * @returns
   */
  _updateWithBuffer(data: Buffer): void {
    const { exports, buffered, mem } = this;
    const length = data.length;
    if (buffered + length < this.chunkSize) {
      data.copy(mem, buffered, 0, length);
      this.buffered += length;
    } else {
      const l = (buffered + length) & ~(this.chunkSize - 1);
      if (l > 65536) {
        let i = 65536 - buffered;
        data.copy(mem, buffered, 0, i);
        exports.update(65536);
        const stop = l - buffered - 65536;
        while (i < stop) {
          data.copy(mem, 0, i, i + 65536);
          exports.update(65536);
          i += 65536;
        }
        data.copy(mem, 0, i, l - buffered);
        exports.update(l - buffered - i);
      } else {
        data.copy(mem, buffered, 0, l - buffered);
        exports.update(l);
      }
      const newBuffered = length + buffered - l;
      this.buffered = newBuffered;
      if (newBuffered > 0) data.copy(mem, 0, length - newBuffered, length);
    }
  }

  digest(type: BufferEncoding) {
    const { exports, buffered, mem, digestSize } = this;
    exports.final(buffered);
    this.instancesPool.push(this);
    const hex = mem.toString('latin1', 0, digestSize);
    if (type === 'hex') return hex;
    if (type === 'binary' || !type) return Buffer.from(hex, 'hex');
    return Buffer.from(hex, 'hex').toString(type);
  }
}

const create = (
  wasmModule: WebAssembly.Module,
  instancesPool: WasmHash[],
  chunkSize: number,
  digestSize: number,
): WasmHash => {
  if (instancesPool.length > 0) {
    const old = instancesPool.pop() as WasmHash;
    old.reset();
    return old;
  }
  return new WasmHash(
    new WebAssembly.Instance(wasmModule),
    instancesPool,
    chunkSize,
    digestSize,
  );
};

export default create;
