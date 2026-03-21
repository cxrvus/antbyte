// @ts-check
/** @import * as AntByte from "../lib" AntByte */

import { writeFileSync } from 'fs'
import { run, size, newWorld, events, randomInt } from "../lib.mjs"

const KEEP_FILES = false;

/** @returns {AntByte.World} */
function generateWorld() {
	const world = newWorld()

	const antCount = randomInt(16) + 5

	for (let i = 0; i <= antCount; i++) {
		world.ants[i] = generateAnt(i)
	}

	return world
}

/** @param {number} index @returns {AntByte.Behavior} */
function generateAnt(index) {
	// todo: just pass probability object
	// manual tweaking...
	const mandatoryInputs = ['T6', 'T7']
	const blockedInputs = [...['R4', 'R5', 'R6', 'R7'], ...mandatoryInputs]
	const filteredInputs = events.input.filter(p => !blockedInputs.includes(p))
	const mandatoryOutputs = ['A0', 'A1', 'AK', 'D0', 'DX']
	const blockedOutputs = [...['A4', 'A5', 'A6', 'A7'], ...mandatoryOutputs]
	const filteredOutputs = events.output.filter(p => !blockedOutputs.includes(p))
	//

	let inputs = getSubset(filteredInputs, randomInt(4) + 2);
	let outputs = getSubset(filteredOutputs, randomInt(4) + 4);

	// todo: automate
	inputs = [...inputs, ...mandatoryInputs]
	outputs = [...outputs, ...mandatoryOutputs]

	// memory alignment
	const inputMem = inputs.filter(x => x.startsWith('M'))
	outputs = outputs.filter(x => !x.startsWith('M') !== inputMem.includes(x))
		.concat(inputMem.filter(x => !outputs.includes(x)))

	const inputCount = inputs.length;
	const outputCount = outputs.length;

	const valueCount = 2 ** inputCount
	const maxValue = 2 ** outputCount + 1

	const logic = []

	for (let i = 0; i < valueCount; i++) logic.push(randomInt(maxValue))

	return { name: index.toString(), outputs, inputs, logic }
}

/** @param {string[]} superSet @param {number} amount @returns {string[]} */
function getSubset(superSet, amount) {
	const set = [...superSet]
	/** @type {string[]} */
	const subset = [];
	let setSize = set.length;

	while (subset.length < amount) {
		const index = randomInt(setSize);
		subset.push(set[index]);
		set.splice(index, 1);
		setSize--;
	}

	return subset
}

const world = generateWorld()

world.cfg = { ...size(128), speed: 2, fps: 12 }

if (KEEP_FILES) {
	const timestamp = new Date().toISOString().slice(0, 19).replace(/:/g, '-').replace('T', '-')
	const dirname = import.meta.dirname;
	writeFileSync(`${dirname}/../../tmp/random_world-${timestamp}.ant.json`, JSON.stringify(world), 'utf-8')
}

run(world)
