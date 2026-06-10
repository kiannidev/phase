import type { CastPaymentMode, GameAction } from "../adapter/types";
import { usePreferencesStore } from "../stores/preferencesStore";

const MANUAL_CAST_PAYMENT_MODE: CastPaymentMode = { type: "Manual" };

export function applySpellPaymentPreference(action: GameAction): GameAction {
  if (usePreferencesStore.getState().spellPaymentMode !== "manual") return action;

  switch (action.type) {
    case "CastSpell":
    case "CastSpellForFree":
    case "CastSpellAsMiracle":
    case "CastSpellAsMadness":
    case "CastSpellAsSneak":
    case "CastSpellAsWebSlinging":
      return {
        ...action,
        data: { ...action.data, payment_mode: MANUAL_CAST_PAYMENT_MODE },
      };
    default:
      return action;
  }
}
