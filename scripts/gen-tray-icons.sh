#!/bin/sh
# Regenerates the tray icons in src-tauri/icons/ from the Octicons GitHub mark
# (MIT, https://github.com/primer/octicons). Requires rsvg-convert (brew install librsvg).
set -eu
cd "$(dirname "$0")/../src-tauri/icons"

MARK='M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z'

# tray <name> <glyph color> [badge color]: 44px icon; the badge knocks a gap out of the mark.
tray() {
  badge=""; mask=""
  if [ -n "${3:-}" ]; then
    mask='mask="url(#m)"'
    badge="<circle cx=\"13\" cy=\"13\" r=\"3.4\" fill=\"$3\"/>"
  fi
  rsvg-convert -w 44 -h 44 -o "$1.png" <<SVG
<svg xmlns="http://www.w3.org/2000/svg" viewBox="-1 -1 18 18">
  <mask id="m" maskUnits="userSpaceOnUse" x="-1" y="-1" width="18" height="18">
    <rect x="-1" y="-1" width="18" height="18" fill="#fff"/>
    <circle cx="13" cy="13" r="4.8" fill="#000"/>
  </mask>
  <path $mask fill="$2" d="$MARK"/>$badge
</svg>
SVG
}

GREEN=#22c55e YELLOW=#eab308 RED=#ef4444 GRAY=#9ca3af
tray tray-healthy '#000'
for theme in light dark; do
  [ $theme = light ] && glyph='#000' || glyph='#fff'
  tray tray-degraded-$theme  $glyph $YELLOW
  tray tray-unhealthy-$theme $glyph $RED
  tray tray-unknown-$theme   $glyph $GRAY
done

# Menu item status dots
for c in green:$GREEN yellow:$YELLOW red:$RED gray:$GRAY; do
  rsvg-convert -w 32 -h 32 -o "dot-${c%%:*}.png" <<SVG
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><circle cx="16" cy="16" r="13" fill="${c#*:}"/></svg>
SVG
done
