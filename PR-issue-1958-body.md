## Summary

- Fix Shared Animosity ([issue #1958](https://github.com/phase-rs/phase/issues/1958)): the "for each other attacking creature that shares a creature type with **it**" clause now binds **it** to the attacking creature (`TriggeringSource`), not the enchantment (`SelfRef`), so the +1/+0 pump scales correctly.
- Thread `ParseContext` through shared-quality parsing (`parse_shared_quality_clause`, `parse_that_clause_suffix`), for-each quantity fallback, pump "for each" stripping, and `thread_for_each_subject` so trigger-subject anaphors resolve during attack triggers.

## Root cause

The for-each filter used `SharesQuality` with reference `SelfRef` (the ability source). At resolution, other attackers were compared to Shared Animosity instead of the attacking creature, yielding a count of zero and no power boost.

## Test plan

- [x] `cargo test -p engine --lib shared_animosity`
- [x] `cargo test -p engine --lib trigger_attacker_it_gets`
- [x] `cargo test -p engine --lib parse_for_each_other_attacking_creature_sharing_type_with_it`
- [x] `cargo test -p engine --lib parse_type_phrase_other_attacking_creature_shares_type_with_it`

Fixes #1958
