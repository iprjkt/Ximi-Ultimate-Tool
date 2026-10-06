#!/usr/bin/env bash
# Rebuilds resources/ximi-pkglabels.jar from XimiPkgLabels.java.
# Needs a JDK (javac, jar) and D8 from the r8 package:
#   curl -o r8.jar https://dl.google.com/android/maven2/com/android/tools/r8/9.5.22/r8-9.5.22.jar
#   R8_JAR=./r8.jar ./build.sh
# After rebuilding, bump LABEL_HELPER_VERSION in src/adb.rs so phones get the new copy.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
r8="${R8_JAR:?set R8_JAR to the path of r8.jar}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

javac --release 8 -nowarn -d "$work/classes" "$here/XimiPkgLabels.java"
# min-api 21 keeps it loadable on Android 5.0+.
java -cp "$r8" com.android.tools.r8.D8 --release --min-api 21 \
    --output "$work" "$work/classes/XimiPkgLabels.class"
(cd "$work" && jar cfM ximi-pkglabels.jar classes.dex)
mkdir -p "$here/../resources"
cp "$work/ximi-pkglabels.jar" "$here/../resources/ximi-pkglabels.jar"
echo "wrote $here/../resources/ximi-pkglabels.jar"
