// @ts-check
/** @import * as AntByte from "../js/lib" AntByte */

import { ant, run } from "../js/lib.mjs"
import { size, bits, byte} from "../js/util.mjs"

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
