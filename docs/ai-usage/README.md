# Uso de IA: modelo, tokens y coste

> Documento vivo. Se actualiza al cerrar cada paso con `node scripts/ai-usage.mjs`.
> Última actualización: 2026-10-04 (cierre de F1).

## 1. Modelo utilizado

| Campo | Valor |
|---|---|
| Herramienta | Claude Code (CLI, Windows 11) |
| Modelo | **Claude Opus 5.5**, `claude-opus-5-5` (todas las llamadas de F0 y F1; sin subagentes) |
| Contexto / salida máx. | 1M tokens / 128K tokens |
| Precio API (USD por millón de tokens) | entrada 4 · salida 20 · escritura en caché (1 h) 8 · lectura de caché 0,20 |

> **Coste equivalente.** Con una suscripción (Pro/Max) no se paga por token, sino que hay límites de uso por periodo. Las cifras en USD de este documento son lo que costaría el mismo uso con la API. Sirven para comparar modelos y dimensionar la licencia, no son una factura.

## 2. Cómo se mide

Claude Code guarda cada sesión en `~/.claude/projects/<proyecto>/<sesión>.jsonl`, con el uso **real** de cada llamada a la API: tokens de entrada, de escritura y lectura de caché, de salida y de razonamiento (*thinking*).

`scripts/ai-usage.mjs` hace tres cosas:
- reparte las llamadas por fase y paso según las ventanas de `phases.json` (cada paso termina cuando se fusiona su PR);
- calcula el coste equivalente;
- simula el mismo uso con otros modelos.

```
node scripts/ai-usage.mjs ~/.claude/projects/C--ProgsConIA-editorMD/<sesión>.jsonl docs/ai-usage/phases.json docs/ai-usage/usage-session1.json
```

## 3. Resultados de F0 y F1 (medidos)

| Fase | Paso | Llamadas | Lect. caché | Salida (razonamiento) | Coste eq. USD | Contexto medio/llamada |
|---|---|--:|--:|--:|--:|--:|
| F0 | Plan + esqueleto (PR #1) | 63 | 9,1 M | 123 k (19 k) | 6,66 | 145 k |
| F0 | Versión 0.0.1 (PR #2) | 13 | 2,7 M | 15 k (2 k) | 0,98 | 207 k |
| F0 | Linters (PR #3) | 22 | 5,1 M | 26 k (2 k) | 1,75 | 230 k |
| F0 | Cierre F0 (PR #4) | 9 | 2,3 M | 18 k (4 k) | 0,99 | 256 k |
| **F0** | **total** | **107** | **19,2 M** | **182 k** | **10,38** | |
| F1 | 1/5 Modelo M01 (PR #5) | 17 | 4,8 M | 25 k (2 k) | 1,68 | 282 k |
| F1 | 2/5 Codec M07 (PR #6) | 14 | 4,3 M | 23 k (4 k) | 1,53 | 310 k |
| F1 | 3/5 Editor Milkdown (PR #7) | 45 | 16,1 M | 62 k (14 k) | 5,06 | 358 k |
| F1 | 4/5 Controlador C01 (PR #8) | 42 | 18,4 M | 63 k (15 k) | 5,54 | 437 k |
| F1 | 5/5 Cierre F1 (PR #9) | 13 | 6,3 M | 16 k (2 k) | 1,73 | 485 k |
| **F1** | **total** | **131** | **49,9 M** | **190 k** | **15,54** | |

**Estimaciones previas:** en F0 y F1 **no se hicieron** estimaciones antes de cada paso; la norma empieza en F2. Como referencia retrospectiva, un paso «pequeño» (PR #2, #4, #5, #6, #9) costó entre 1 y 1,8 USD eq. con 9-17 llamadas, y uno «grande» (PR #7, #8) entre 5 y 5,5 USD eq. con 42-45 llamadas.

### Lo que enseñan los datos

1. **El coste lo domina la relectura del contexto, no la escritura de código.** El 98,6 % de los tokens son lecturas de caché: en cada llamada se reenvía toda la conversación. En coste, las lecturas suponen alrededor del 53 %, la salida el 28 % y las escrituras de caché el 19 %.
2. **El contexto crece con la sesión.** El contexto medio por llamada ha pasado de 145 k a 485 k tokens, así que la misma tarea cuesta unas 3 veces más al final de una sesión larga que al principio.
   → **Medida inmediata: empezar cada fase en una sesión nueva.** CONTEXT.md, STATUS.md y PLAN.md existen precisamente para eso.
3. **El razonamiento interno (*thinking*) es pequeño:** unos 68 k de los 377 k tokens de salida (18 %).

## 4. RQ.1: ¿modelo caro con poco *rework* o modelo barato con más *rework*?

### 4.1 Lo que ya se puede afirmar con los datos de esta sesión

**(a) La diferencia de precio en trabajo agéntico es menor de lo que parece.** Opus 5.5 cuesta el doble que Sonnet 5.5 por token de entrada y de salida, pero **la lectura de caché cuesta lo mismo** (0,20 USD/M). Con el mismo volumen de tokens, esta sesión habría costado:

| Mismo uso con… | Coste eq. | vs. real |
|---|--:|--:|
| Fable 5.1 | 49,68 | 184 % |
| **Opus 5.5 (real)** | **27,04** | 100 % |
| Sonnet 5.5 | 20,68 | 77 % |
| Haiku 4.5 | 10,34 | 38 % ⚠ |

→ Sonnet 5.5 solo ahorra un **23 %** a igualdad de tokens. Como el coste crece con el número de llamadas (cada una relee el contexto), **si Sonnet necesitara más de ~1,3 veces las llamadas de Opus para el mismo resultado, Opus sería más barato**.

⚠ **Haiku 4.5 no podría haber hecho esta sesión:** su contexto máximo es de 200 k tokens y aquí se llegó a ~500 k. Tendría que trabajar en sesiones mucho más cortas, con más relectura de documentación y más riesgo de perder contexto.

**(b) Calidad observada de Opus 5.5 en F0-F1** (indicadores objetivos):

| Indicador | Valor |
|---|---|
| PR con la CI en verde al primer intento | **9 de 9** (4 SO + lint) |
| PR con cambios pedidos en la revisión humana | 0 de 9 |
| Defectos conocidos que llegaron a `main` | 0 |
| Pruebas | 341 TS + 4 Rust; Modelo, Controlador y adaptadores al 100 % de líneas |
| *Rework* por errores propios, detectados antes del commit (estimación a partir del historial) | ≈ 18 de 242 llamadas (≈ 7 %) |
| Llamadas en incidencias externas (el `*` causado por una edición manual) | ≈ 14 (≈ 6 %) |

Errores propios registrados (todos detectados por las herramientas antes de llegar a un PR):
- 3 fallos al escribir *scripts* con *heredocs* en la shell;
- una conversión de tipos sin sentido en V02;
- una sintaxis no válida (`?.` con campo privado);
- 5 errores de lint (`prefer-const`);
- 11 avisos de TypeDoc por `{@inheritDoc}` y 2 errores de tipos implícitos;
- un recuento erróneo en un mensaje de commit («26» en lugar de 22 pruebas);
- una falsa alarma de cobertura;
- una verificación hecha con un fichero desactualizado.

El *rework* de Opus 5.5 aquí es bajo y barato, porque las herramientas (tests, lint, CI) lo detectan pronto. **Esta es la variable clave de la RQ.1:** el coste de un modelo peor depende de cuántos de sus errores se escapan de esas redes, no solo de cuántos comete.

### 4.2 Diseño del experimento (propuesto, pendiente de aprobación)

**Hipótesis.** H₀: el coste total por tarea terminada de Sonnet 5.5 es menor o igual que el de Opus 5.5. H₁: es mayor.

**Coste total por tarea**:

C = Σ tokens × precio + minutos de revisión humana × valor/hora + coste de los defectos que se escapan.

**Diseño:**
1. **Tareas:** 2 pasos reales de F2 con especificación cerrada y criterios de aceptación objetivos (pruebas ocultas + lint + CI). Por ejemplo, «M09 TextCodec: detección de codificación» y «FileService: escritura atómica».
2. **Tratamientos:** el mismo *prompt* ejecutado por **Opus 5.5** y por **Sonnet 5.5**, cada uno en un subagente con su propio *git worktree* (aislado) y sesión nueva, con 2-3 repeticiones por modelo para estimar la varianza. Haiku 4.5 es opcional y queda limitado por su contexto.
3. **Métricas:**
   - tokens y coste equivalente (con `ai-usage.mjs`);
   - número de llamadas;
   - ciclos de *rework* (pruebas o lint en rojo → corrección);
   - pruebas ocultas superadas;
   - defectos encontrados en tu revisión (checklist fija);
   - minutos de tu revisión;
   - métricas de código (cobertura, avisos, complejidad).
4. **Regla de decisión:** se elige el modelo con menor C. Si las diferencias son menores que la varianza, el más barato.
5. **Validez:**
   - mismas herramientas y mismo *effort* en ambos modelos;
   - las tareas se asignan sin saber qué modelo las hará;
   - tú revisas **sin saber** qué modelo produjo cada PR (revisión ciega);
   - se descartan las incidencias externas (antivirus, interacción manual), como pide la pregunta.

**Coste estimado del experimento:** unos 10-25 USD eq. (2 tareas × 2 modelos × 2-3 repeticiones, en sesiones cortas). El código que gane se integra en F2, así que parte de ese coste no se pierde.

## 5. Norma a partir de F2

En cada paso: **estimación previa** (llamadas y USD eq., por analogía con la tabla de la §3) → **medición real** al fusionar el PR → registro aquí y en la web. Cada fase empieza en una sesión nueva.
