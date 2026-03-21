// @ts-check
/** @import * as AntByte from "../js/lib" AntByte */

import { ant, run,  size, bits, byte } from "../js/lib.mjs"

run({
	cfg: {
		...size(32),
		// ...
	},
	ants: {
		1: ant("main", () => {
			// ...
			return {
				// ...
			}
		}),
	},
})
