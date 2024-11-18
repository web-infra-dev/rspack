import { fiftyK } from "./50k";
import { fiftyK as fiftyK_1 } from "./50k-1";
import { fiftyK as fiftyK_2 } from "./50k-2";
import { fiftyK as fiftyK_3 } from "./50k-3";
import { fiftyK as fiftyK_4 } from "./50k-4";
import { fiftyK as fiftyK_5 } from "./50k-5";
import { fiftyK as fiftyK_6 } from "./50k-6";
import { fiftyK as fiftyK_7 } from "./50k-7";
import { fiftyK as fiftyK_8 } from "./50k-8";

import {
  fiftyK1,
  fiftyK2,
  fiftyK3,
  fiftyK4,
  fiftyK5,
  fiftyK6,
  fiftyK7,
  fiftyK8,
} from "./400k";

window.lib = {
  fiftyK,
  fiftyK1,
  fiftyK2,
  fiftyK3,
  fiftyK4,
  fiftyK5,
  fiftyK6,
  fiftyK7,
  fiftyK8,
  fiftyK_1,
  fiftyK_2,
  fiftyK_3,
  fiftyK_4,
  fiftyK_5,
  fiftyK_6,
  fiftyK_7,
  fiftyK_8,
};

it('should ensure max size fit', () => {
	const stats = __non_webpack_require__('./stats.json');

	const chunks = new Map();

	for (const c of stats.children[0].chunks) {
		chunks.set(c.id, c)
	}

	expect(chunks.size).toBe(4)

	expect(chunks.get('main~1')).toBeDefined()
	expect(chunks.get('main~1').modules.length).toBe(3)
	expect(chunks.get('main~2')).toBeDefined()
	expect(chunks.get('main~2').modules.length).toBe(3)
	expect(chunks.get('main~3')).toBeDefined()
	expect(chunks.get('main~3').modules.length).toBe(1)
})
