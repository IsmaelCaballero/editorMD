#!/bin/bash
# Evalúa una ejecución del experimento RQ.1 sin modificar su rama (F2 paso 1, tarea T1).
# Uso: scripts/rq1-eval.sh <id del agente> <fichero de pruebas ocultas>
# El worktree del agente debe existir en .claude/worktrees/agent-<id> (solo en la máquina donde se ejecutó).
W=$(git rev-parse --show-toplevel)/.claude/worktrees/agent-$1; T=$2; M=$W/src-tauri/Cargo.toml
cd $W || exit 1
echo "== $1 rama: $(git branch --list 'rq1/*' --points-at HEAD) HEAD $(git log --oneline -1)  limpio: $(git status --short | wc -l) cambios"
echo "commits: $(git log --oneline 01b96b1..HEAD | wc -l)"
cargo fmt --manifest-path $M --check >/dev/null 2>&1 && echo "fmt: ok" || echo "fmt: FALLA"
cargo clippy --manifest-path $M --all-targets -- -D warnings >/dev/null 2>&1 && echo "clippy: ok" || echo "clippy: FALLA"
echo "tests propios: $(cargo test --manifest-path $M 2>&1 | grep 'test result' | sed 's/; 0 ignored.*//' | tr '\n' ' ')"
mkdir -p $W/src-tauri/tests && cp "$T" $W/src-tauri/tests/hidden.rs
echo "ocultas: $(cargo test --manifest-path $M --test hidden 2>&1 | grep -E 'test result|^test .*FAILED' | tr '\n' ' ')"
rm $W/src-tauri/tests/hidden.rs; rmdir $W/src-tauri/tests 2>/dev/null
echo "líneas text_codec.rs: $(wc -l < $W/src-tauri/src/model/text_codec.rs) · unwrap/expect fuera de tests: $(sed '/#\[cfg(test)\]/,$d' $W/src-tauri/src/model/text_codec.rs | grep -cE '\.(unwrap|expect)\(')"
echo "estado final: $(git status --short | wc -l) cambios"
