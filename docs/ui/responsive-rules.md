# Responsive Rules

The shell has three layout bands. Compact layouts below 640 px use safe-area padding and bottom navigation. Medium layouts from 640 through 1023 px use a top bar. Wide layouts from 1024 px use a persistent left rail. Study mode hides global navigation at every width.

Layout also exposes pointer capability through `data-input`. Touch-capable compact controls must be at least 44 by 44 pixels. Content must reflow without horizontal page scrolling at every required viewport, at 200% text zoom, and with both English and Simplified Chinese text.

Dataset master/detail content becomes a vertical flow on compact screens. Statistics become three compact columns and mistake filters remain horizontally scrollable chips. Safe-area insets are applied to the application shell and compact navigation.
