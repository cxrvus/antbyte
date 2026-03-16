// todo: only generate useful bindings

export type { Behavior as BehaviorJSON } from "./bindings/Behavior";
export type { BorderMode } from "./bindings/BorderMode";
export type { ColorMode } from "./bindings/ColorMode";
export type { Event } from "./bindings/Event";
export type { EventBit } from "./bindings/EventBit";
export type { StartingPos } from "./bindings/StartingPos";
export type { TruthTable } from "./bindings/TruthTable";
export type { WorldProperties } from "./bindings/WorldProperties";
import type { WorldConfig as WorldConfigFull } from "./bindings/WorldConfig";

export type WorldConfig = Partial<WorldConfigFull>

export type World = {
	cfg: WorldConfig,
	ants: Behaviors,
}

export type Behaviors = { [key: number]: Behavior }

export type Behavior = {
	name: string
	logic: number[]
	inputs: string[]
	outputs: string[]
}
