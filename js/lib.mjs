// @ts-check
/** @typedef {import("./lib").Behavior} Behavior */
/** @typedef {import("./lib").World} World */

export * from './random.mjs'

import PINS from './bindings/PINS.json' with { type: 'json' }
export { PINS }

/** @param {World} world */
export function run(world) {
	console.log(JSON.stringify(world, null, '\t'))
}

/** @returns {import('./lib').World} */
export const newWorld = () => ({ name: null, cfg: {}, ants: {} })

/** @param {number} value */
export const size = value => ({ height: value, width: value})
