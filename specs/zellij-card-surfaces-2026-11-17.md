# Independent card surfaces — 2026-11-17

Follow-up to the Kie theme review. Feature and pricing cards paint their own
page-colored surface, but the surrounding section's inverted/accent foreground
can reach card headings, prices, feature lists and ghost actions. The corrected
shared CSS anchors card text to its own foreground; section headings outside
cards retain their section color. Theme-specific button styling remains active.

Hosted browser acceptance clones the actual exported cards under default,
inverted, accent and gradient sections, in light/dark and desktop/mobile views.
Every card retains its default foregrounds; headings, prices, lists and linked
features resolve the card's foreground token. The measured matrices are retained.
All existing template screenshots must retain their reviewed appearance.

Verification pending. No local browser/container is used; public rc.9 is unchanged.
