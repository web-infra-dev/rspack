import { Readable } from 'node:stream';
import type { JsAssetSource } from '@rspack/binding';
import { Source } from 'webpack-sources';

/** A final asset stored as Zstandard, with streaming reads for file writes and uploads. */
export class AssetSource extends Source {
  constructor(private readonly native: JsAssetSource) {
    super();
  }

  /** HTTP representation for clients accepting Content-Encoding: zstd. */
  get compressedBuffer(): Buffer {
    return this.native.compressedBuffer;
  }

  get contentHash(): string {
    return this.native.contentHash;
  }

  source(): string {
    return this.native.source();
  }

  buffer(): Buffer {
    return this.native.buffer();
  }

  size(): number {
    return this.native.size();
  }

  map(): null {
    return null;
  }

  updateHash(hash: Parameters<Source['updateHash']>[0]): void {
    hash.update('AssetSource');
    hash.update(this.contentHash);
  }

  /** Each stream owns a decoder and honours downstream backpressure. */
  createReadStream(): Readable {
    const reader = this.native.reader();
    return new Readable({
      read(size) {
        try {
          this.push(reader.read(size));
        } catch (error) {
          this.destroy(error as Error);
        }
      },
      destroy(error, callback) {
        reader.close();
        callback(error);
      },
    });
  }
}
