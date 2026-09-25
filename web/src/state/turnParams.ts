// The turn-param builders (WEB-002): pure functions that compose the
// per-turn execution choices (model / effort / working directory / skill
// references) onto the protocol's own turn/start params — exactly the
// generated TurnStartParams shape, nothing invented.

import type { TurnStartParams, UserInput } from "../protocol/generated";

/** The user's model/provider selection for the next turn. */
export interface ModelChoice {
  /** A model id, or null to use the agent runtime's default. */
  model: string | null;
  /** A reasoning effort, or null to use the runtime's default. */
  effort: string | null;
}

export const NO_MODEL_CHOICE: ModelChoice = { model: null, effort: null };

export function sameModelChoice(
  a: ModelChoice | null | undefined,
  b: ModelChoice | null | undefined,
): boolean {
  return (a?.model ?? null) === (b?.model ?? null) && (a?.effort ?? null) === (b?.effort ?? null);
}

/** Builds a protocol turn/start params object riding the current choices. */
export function buildTurnStart(
  threadId: string,
  input: UserInput[],
  choice: ModelChoice | null,
  cwd: string | null,
): TurnStartParams {
  const params: TurnStartParams = { threadId, input };
  if (choice !== null) {
    if (choice.model !== null && choice.model.trim() !== "") {
      params.model = choice.model.trim();
    }
    if (choice.effort !== null && choice.effort.trim() !== "") {
      params.effort = choice.effort.trim();
    }
  }
  if (cwd !== null && cwd.trim() !== "") {
    params.cwd = cwd.trim();
  }
  return params;
}

/** Builds the input array for a skill reference (with an optional note). */
export function buildSkillReferenceInput(
  name: string,
  path: string,
  note: string | null,
): UserInput[] {
  const input: UserInput[] = [{ type: "skill", name, path }];
  if (note !== null && note.trim() !== "") {
    input.push({ type: "text", text: note.trim() });
  }
  return input;
}
