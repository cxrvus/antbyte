// @ts-check
/** @typedef {import("../js/lib").Behavior} Behavior */
/** @typedef {import("../js/lib").World} World */

import * as util from './util.mjs'

export * from './util.mjs'
export * from './random.mjs'

/**
 * @param {string} name
 * @param {(...args: boolean[]) => Record<string, boolean>} func
 * @returns {Behavior}
 */
export function ant(name, func) {
	const inputs = func.toString().match(/\(([^)]*)\)/)?.[1].split(',').map(p => p.trim()).filter(p => p)

	if (!inputs) throw "malformed function parameters"

	for (const name of inputs) {
		if (!/^[A-Z][A-Z0-9]*(_[A-Z0-9]+)*$/.test(name)) {
			throw `malformed function parameter: "${name}"`
		}
	}

	const inputCount = inputs.length
	const entryCount = 2 ** inputCount
	
	/** @type {Record<string, boolean>[]} */
	const outputRecords = [];

	/** @type {string[]} */
	const outputs = [];

	for (let i = 0; i < entryCount; i++) {
		let inputBits = util.bits(inputCount, i)
		let outputRecord = func(...inputBits)
		outputRecords.push(outputRecord)

		Object.keys(outputRecord).forEach(key => {
			if (!outputs.includes(key)) {
				outputs.push(key)
			}
		})
	}

	/** @type {number[]} */
	const outputValues = [];

	for (const outputRecord of outputRecords) {
		const outputBits = outputs.map(output => outputRecord[output] ?? false)
		outputValues.push(util.byte(...outputBits));
	}

	return { name, inputs, outputs, logic: outputValues }
}

/** @param {World} world */
export function run(world) {
	console.log(JSON.stringify(world, null, '\t'))
}
