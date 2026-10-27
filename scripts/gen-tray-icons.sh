#!/bin/sh
# Regenerates the tray and menu icons in src-tauri/icons/ from the Octicons GitHub mark
# (MIT, https://github.com/primer/octicons). Requires rsvg-convert (brew install librsvg).
set -eu
cd "$(dirname "$0")/../src-tauri/icons"

MARK='M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z'

# Tray icons are template images (macOS tints them to match the menubar, which follows the
# wallpaper rather than the system theme), so status is a badge shape, not a color:
#   degraded "!" | unhealthy "x" | unknown ring. The badge knocks a gap out of the mark.
tray() { # tray <name> [badge symbol markup]
  badge=""; mask=""
  if [ -n "${2:-}" ]; then
    mask='mask="url(#m)"'
    badge="<circle cx=\"12.5\" cy=\"12.5\" r=\"4.2\" mask=\"url(#b)\"/>"
  fi
  rsvg-convert -w 44 -h 44 -o "$1.png" <<SVG
<svg xmlns="http://www.w3.org/2000/svg" viewBox="-1 -1 18 18">
  <mask id="m" maskUnits="userSpaceOnUse" x="-1" y="-1" width="18" height="18">
    <rect x="-1" y="-1" width="18" height="18" fill="#fff"/>
    <circle cx="12.5" cy="12.5" r="5.4" fill="#000"/>
  </mask>
  <mask id="b" maskUnits="userSpaceOnUse" x="-1" y="-1" width="18" height="18">
    <rect x="-1" y="-1" width="18" height="18" fill="#fff"/>
    ${2:-}
  </mask>
  <path $mask d="$MARK"/>$badge
</svg>
SVG
}

tray tray-healthy
tray tray-degraded  '<rect x="11.9" y="9.7" width="1.2" height="3.4" rx=".5"/><circle cx="12.5" cy="14.9" r=".8"/>'
tray tray-unhealthy '<path d="M10.9 10.9L14.1 14.1M14.1 10.9L10.9 14.1" stroke="#000" stroke-width="1.3" stroke-linecap="round"/>'
tray tray-unknown   '<circle cx="12.5" cy="12.5" r="2" fill="#000"/>'

GREEN=#22c55e YELLOW=#eab308 RED=#ef4444 GRAY=#9ca3af
# Menu item status dots
for c in green:$GREEN yellow:$YELLOW red:$RED gray:$GRAY; do
  rsvg-convert -w 32 -h 32 -o "dot-${c%%:*}.png" <<SVG
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><circle cx="16" cy="16" r="13" fill="${c#*:}"/></svg>
SVG
done
