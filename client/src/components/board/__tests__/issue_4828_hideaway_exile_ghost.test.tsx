import { cleanup, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { GameObject, GameState } from "../../../adapter/types.ts";
import { useGameStore } from "../../../stores/gameStore.ts";
import { BoardInteractionContext } from "../BoardInteractionContext.tsx";
import { PermanentCard } from "../PermanentCard.tsx";

vi.mock("../../card/CardImage.tsx", () => ({
  CardImage: ({ cardName, faceDown }: { cardName: string; faceDown?: boolean }) => (
    <div
      aria-label={faceDown ? "Face-down card" : cardName}
      data-face-down={faceDown ? "true" : "false"}
    />
  ),
}));

function makeObject(overrides: Partial<GameObject> = {}): GameObject {
  return {
    id: 1,
    card_id: 100,
    owner: 0,
    controller: 0,
    zone: "Battlefield",
    tapped: false,
    face_down: false,
    flipped: false,
    transformed: false,
    damage_marked: 0,
    dealt_deathtouch_damage: false,
    attached_to: null,
    attachments: [],
    counters: {},
    name: "Host",
    power: null,
    toughness: null,
    loyalty: null,
    card_types: { supertypes: [], core_types: ["Land"], subtypes: [] },
    mana_cost: { type: "NoCost", shards: [], generic: 0 },
    keywords: [],
    abilities: [],
    trigger_definitions: [],
    replacement_definitions: [],
    static_definitions: [],
    color: [],
    base_power: null,
    base_toughness: null,
    base_keywords: [],
    base_color: [],
    timestamp: 1,
    entered_battlefield_turn: null,
    ...overrides,
  };
}

function makeState(objects: GameObject[], exileLinks: NonNullable<GameState["exile_links"]>): GameState {
  return {
    active_player: 0,
    priority_player: 0,
    players: [
      {
        id: 0,
        life: 20,
        poison_counters: 0,
        mana_pool: { mana: [] },
        library: [],
        hand: [],
        graveyard: [],
        has_drawn_this_turn: false,
        lands_played_this_turn: 0,
        turns_taken: 0,
      },
      {
        id: 1,
        life: 20,
        poison_counters: 0,
        mana_pool: { mana: [] },
        library: [],
        hand: [],
        graveyard: [],
        has_drawn_this_turn: false,
        lands_played_this_turn: 0,
        turns_taken: 0,
      },
    ],
    objects: Object.fromEntries(objects.map((o) => [o.id, o])),
    battlefield: objects.filter((o) => o.zone === "Battlefield").map((o) => o.id),
    exile: objects.filter((o) => o.zone === "Exile").map((o) => o.id),
    stack: [],
    combat: null,
    exile_links: exileLinks,
    waiting_for: { type: "Priority", data: { player: 0 } },
  } as unknown as GameState;
}

describe("PermanentCard Hideaway exile ghost (issue #4828)", () => {
  afterEach(() => cleanup());

  beforeEach(() => {
    window.matchMedia = ((query: string) => ({
      matches: query === "(any-hover: hover)",
      media: query,
      onchange: null,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })) as unknown as typeof window.matchMedia;
  });

  it("hides an opponent's Hideaway-exiled card on the source permanent's exile ghost", () => {
    const source = makeObject({
      id: 1,
      owner: 1,
      controller: 1,
      name: "Mosswort Bridge",
    });
    const hidden = makeObject({
      id: 2,
      owner: 1,
      controller: 1,
      zone: "Exile",
      name: "Ghalta, Primal Hunter",
      face_down: true,
    });
    const gameState = makeState(
      [source, hidden],
      [{ exiled_id: 2, source_id: 1, kind: "HideawayLookable" }],
    );
    useGameStore.setState({ gameState, waitingFor: gameState.waiting_for });

    const { queryByLabelText } = render(
      <BoardInteractionContext.Provider
        value={{
          activatableObjectIds: new Set(),
          boardChoiceObjectIds: new Set(),
          committedAttackerIds: new Set(),
          incomingAttackerCounts: new Map(),
          manaTappableObjectIds: new Set(),
          selectableSacrificeObjectIds: new Set(),
          selectableManaCostCreatureIds: new Set(),
          undoableTapObjectIds: new Set(),
          validAttackerIds: new Set(),
          validTargetObjectIds: new Set(),
        }}
      >
        <PermanentCard objectId={1} />
      </BoardInteractionContext.Provider>,
    );

    expect(queryByLabelText("Face-down card")).not.toBeNull();
    expect(queryByLabelText("Ghalta, Primal Hunter")).toBeNull();
  });
});
