#!/usr/bin/env bash
# CIの複数のジョブに記述したツールの版が、揃っていることを確認する。Gitのフック（prek）とCIで実行する。
#
# - taiki-e/install-actionのtoolに記述したツールには、すべて版（<ツール>@X.Y.Z）を指定する
# - 同じツールの版は、すべてのジョブで同じにする（例: protoc）
# - j178/prek-actionのprek-versionは、すべてのジョブで同じにする
#
# Dependabotは、これらの版を更新しないため、手動で更新するときの食い違いを、このスクリプトで検出する。

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

ci=.github/workflows/ci.yml
failed=0

# taiki-e/install-actionのtoolの値を、1行に1つの「ツール@版」に分ける。
tools=$(sed -nE 's/^ *tool: (.+)$/\1/p' "${ci}" | tr ',' '\n' | sed -E 's/^ +//; s/ +$//')

while IFS= read -r tool; do
    [[ -z "${tool}" ]] && continue
    if [[ "${tool}" != *@* ]]; then
        echo "error: ${tool} in ${ci} has no version; specify it as ${tool}@X.Y.Z" >&2
        failed=1
    fi
done <<<"${tools}"

# ツールの名前ごとに、記述した版の種類を数える。
while IFS= read -r name; do
    [[ -z "${name}" ]] && continue
    versions=$(grep -E "^${name}@" <<<"${tools}" | cut -d@ -f2 | sort -u)
    if [[ $(wc -l <<<"${versions}") -ne 1 ]]; then
        echo "error: the versions of ${name} in ${ci} differ ($(tr '\n' ' ' <<<"${versions}")); set them to the same version" >&2
        failed=1
    fi
done < <(grep '@' <<<"${tools}" | cut -d@ -f1 | sort -u)

prek_versions=$(sed -nE 's/^ *prek-version: (.+)$/\1/p' "${ci}" | sort -u)
if [[ -z "${prek_versions}" ]]; then
    echo "error: could not read prek-version from ${ci}" >&2
    failed=1
elif [[ $(wc -l <<<"${prek_versions}") -ne 1 ]]; then
    echo "error: the versions of prek in ${ci} differ ($(tr '\n' ' ' <<<"${prek_versions}")); set them to the same version" >&2
    failed=1
fi

exit "${failed}"
