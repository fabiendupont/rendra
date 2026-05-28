#!/bin/bash
# Concatenates all CSS source files into a combined output
cd "$(dirname "$0")"

cat src/base/reset.css src/base/tokens.css src/base/themes/dark.css src/base/themes/light.css src/base/typography.css \
    src/layouts/*.css src/components/*.css > rendra-ui.css

echo "Built rendra-ui.css ($(wc -c < rendra-ui.css) bytes)"

cp src/js/rendra.js rendra-ui.js
echo "Built rendra-ui.js ($(wc -c < rendra-ui.js) bytes)"
