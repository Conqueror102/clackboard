# LockOverlay

The full-screen view shown while the keyboard is locked. No card: one quiet column in the middle of a plain `surface`, and the way out pinned to the bottom.

**Order, top to bottom (fixed):** a small status line "Keyboard locked" with a clay dot → the time left in the `ui` face at weight 200 with a 2 px clay hairline under it (when duration is Never, "Unlock when you're done" instead) → up to 6 recent keys as plain characters, no chips, the newest at full strength and older ones fading out (only if "Show the keys you press" is on) → at the bottom, HoldButton as a soft filled pill with no border, and one hint line with the escape shortcut as plain glyphs.

**Colour:** clay only on the status dot, the progress line, the newest key for a moment and the hold fill. The time and the line turn `caution` in the last 10 s.

**Rules:**
- The overlay covers every display, on top of all windows, but it is NOT a security lock screen. No password; the pointer stays live.
- The way out is always on screen: HoldButton, the escape shortcut, and the timer when one is set.
- Optional dim: when "Dim the screen while locked" is on, lower brightness or draw the overlay at full opacity so smudges show up better on a dark screen. Never turn the display off; the user needs to see the exit.
- Fade in and out over `motion-slow`; no other motion.
- On unlock, show a 2 s toast "Keyboard unlocked" in the main window, then close the overlay.

**Copy:** no headline. The status line says "Keyboard locked"; the hint says "or press" followed by the shortcut. Never jokes about danger or data.
