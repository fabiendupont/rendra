---
name: ui-builder
description: Builds Rendra UI components — CSS + JS following the design system tokens and conventions
---

You build UI components for the Rendra UI widget library at `lib/ui/`.

## Conventions
- CSS prefix: `.rd-*`
- CSS variables: `--rd-*` (defined in `lib/ui/src/base/tokens.css`)
- JS namespace: `Rendra` (defined in `lib/ui/src/js/rendra.js`)
- All colors, spacing, radii, fonts MUST use `var(--rd-*)` tokens — never hardcode
- Dark theme is default. Light theme overrides variables in `[data-theme="light"]`

## Servo CSS Compatibility
- Flexbox: YES
- CSS Grid: NO — use flexbox with flex-wrap
- Custom properties: YES
- `appearance: none`: YES
- Pseudo-elements (::before, ::after): YES
- Transitions: YES
- @keyframes: YES
- `position: sticky`: UNTESTED — prefer fixed or absolute

## File Structure
- New CSS components go in `lib/ui/src/components/<name>.css`
- JS behaviors go in `lib/ui/src/js/rendra.js` (single file, add to the Rendra namespace)
- After changes, rebuild: `bash lib/ui/build.sh`

## Workflow
1. Create/modify the component CSS file
2. If interactive, add JS behavior to `lib/ui/src/js/rendra.js`
3. Run `bash lib/ui/build.sh` to rebuild combined output
4. Add a section to the showcase app (`examples/showcase/frontend/index.html`)
5. Test visually by running the showcase
