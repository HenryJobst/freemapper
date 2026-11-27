#!/usr/bin/env bash
# Builds mapper-inspect against a checkout of OpenOrienteering Mapper.
#
#   tools/mapper-inspect/build.sh <mapper-checkout> <build-dir> [qt5-prefix]
#
# Needs CMake, Ninja, Qt 5 (Core, Widgets, LinguistTools), PROJ and a C++ compiler.
# The checkout is copied; it is not changed. The binary lands in
# <build-dir>/build/src/mapper-inspect.
set -euo pipefail

mapper=${1:?usage: build.sh <mapper-checkout> <build-dir> [qt5-prefix]}
out=${2:?usage: build.sh <mapper-checkout> <build-dir> [qt5-prefix]}
qt=${3:-/opt/homebrew/opt/qt@5}
here=$(cd "$(dirname "$0")" && pwd)

mkdir -p "$out"
out=$(cd "$out" && pwd)
rm -rf "$out/src"
cp -R "$mapper" "$out/src"

# Not needed for the importer: manual (needs Doxygen), packaging, licence and API docs.
sed -E -i.bak 's#^add_subdirectory\("(doc/manual|packaging|doc/licensing|doc/api|packaging/src)"\)#\#&#' "$out/src/CMakeLists.txt"
rm -f "$out/src/CMakeLists.txt.bak"

cp "$here/mapper_inspect.cpp" "$out/src/src/mapper_inspect.cpp"
cat >> "$out/src/src/CMakeLists.txt" <<'CMAKE'

add_executable(mapper-inspect mapper_inspect.cpp)
target_link_libraries(mapper-inspect Mapper_Common)
CMAKE

cmake -G Ninja -S "$out/src" -B "$out/build" \
  -DCMAKE_POLICY_VERSION_MINIMUM=3.5 \
  -DCMAKE_PREFIX_PATH="$qt" \
  -DCMAKE_BUILD_TYPE=Release \
  -DMapper_USE_GDAL=OFF -DMapper_WITH_COVE=OFF -DBUILD_TESTING=OFF
ninja -C "$out/build" mapper-inspect
echo "built: $out/build/src/mapper-inspect"
