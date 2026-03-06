// @ts-check
/** @typedef {import("../js/lib").Behavior} Behavior */

import { toBits, toByte } from './util.mjs';

/**
 * @param {string} name
 * @param {(...args: boolean[]) => Record<string, boolean>} func
 * @returns {Behavior}
 */
export function ant(name, func) {
	const inputs = func.toString().match(/\(([^)]*)\)/)?.[1].split(',').map(p => p.trim())

	if (!inputs) throw "malformed function parameters"

	for (const name of inputs) {
		if (!/^[A-Z][A-Z0-9]*(_[A-Z0-9]+)*$/.test(name)) {
			throw `malformed parameter: "${name}"`
		}
	}

	const inputCount = inputs.length
	const entryCount = 2 ** inputCount
	
	/** @type {Record<string, boolean>[]} */
	const outputRecords = [];

	/** @type {string[]} */
	const outputs = [];

	for (let i = 0; i < entryCount; i++) {
		let inputBits = toBits(inputCount, i)
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
		outputValues.push(toByte(...outputBits));
	}

	return { name, inputs, outputs, logic: outputValues }
}
