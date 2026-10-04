#!/bin/bash
# Evalúa una ejecución del experimento RQ.1 sin modificar su rama (tareas T1 y T2).
# Uso: scripts/rq1-eval.sh <id del agente> <fichero de pruebas ocultas> [commit base] [fichero del módulo]
#   T1: scripts/rq1-eval.sh <id> T1-hidden-tests.rs 01b96b1 src-tauri/src/model/text_codec.rs   (valores por defecto)
#   T2: scripts/rq1-eval.sh <id> T2-hidden-tests.rs c5f2feb src-tauri/src/model/file_service.rs
# El worktree del agente debe existir en .claude/worktrees/agent-<id> (solo en la máquina donde se ejecutó).
W=$(git rev-parse --show-toplevel)/.claude/worktrees/agent-$1; T=$2; B=${3:-01b96b1}; F=${4:-src-tauri/src/model/text_codec.rs}
M=$W/src-tauri/Cargo.toml
cd $W || exit 1
echo "== $1 rama: $(git branch --list 'rq1/*' --points-at HEAD) HEAD $(git log --oneline -1)  limpio: $(git status --short | wc -l) cambios"
echo "commits: $(git log --oneline $B..HEAD | wc -l)"
cargo fmt --manifest-path $M --check >/dev/null 2>&1 && echo "fmt: ok" || echo "fmt: FALLA"
cargo clippy --manifest-path $M --all-targets -- -D warnings >/dev/null 2>&1 && echo "clippy: ok" || echo "clippy: FALLA"
echo "tests propios: $(cargo test --manifest-path $M 2>&1 | grep 'test result' | sed 's/; 0 ignored.*//' | tr '\n' ' ')"
mkdir -p $W/src-tauri/tests && cp "$T" $W/src-tauri/tests/hidden.rs
echo "ocultas: $(cargo test --manifest-path $M --test hidden 2>&1 | grep -E 'test result|^test .*FAILED|^error' | tr '\n' ' ')"
rm $W/src-tauri/tests/hidden.rs; rmdir $W/src-tauri/tests 2>/dev/null
L=$(cat $W/$F $W/${F%.rs}/*.rs 2>/dev/null | wc -l)
U=$(for f in $W/$F $W/${F%.rs}/*.rs; do [ -f $f ] && [ "$(basename $f)" != tests.rs ] && sed '/#\[cfg(test)\]/,$d' $f; done | grep -v '^ *//' | grep -cE '\.(unwrap|expect)\(')
echo "líneas $(basename $F) (+ submódulos): $L · unwrap/expect fuera de tests: $U"
echo "estado final: $(git status --short | wc -l) cambios"
