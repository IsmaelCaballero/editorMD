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

### 4.2 Diseño del experimento (aprobado el 2026-10-04, con 3 repeticiones por modelo)

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

### 4.3 Ejecución

| Tarea | Paso de F2 | Especificación | Ejecuciones | Estado |
|---|---|---|---|---|
| T1 | 1/8 M09 detección y decodificación | [`rq1/T1-text-codec-decode.md`](rq1/T1-text-codec-decode.md) | r1-r6 (3 Opus 5.5 + 3 Sonnet 5.5, asignación aleatoria y oculta) | ✅ terminada (§4.4) |
| T2 | 4/8 M08 FileService | [`rq1/T2-file-service.md`](rq1/T2-file-service.md) | r1-r6 (3 Opus 5.5 + 3 Sonnet 5.5, asignación aleatoria y oculta) | ✅ terminada (§4.5) |

**Procedimiento:**
1. El orquestador (Opus 5.5, sesión principal) escribe la especificación (es el encargo común) y unas **pruebas de aceptación ocultas** que no están en el repositorio hasta la evaluación.
2. Se lanzan 6 subagentes en paralelo, cada uno en su *git worktree*, con el mismo encargo; solo cambia la etiqueta de la rama (`rq1/T<n>/r<k>`). Cada ejecución empieza con el contexto vacío.
3. Evaluación automática: pruebas ocultas, `cargo test`, clippy, rustfmt y métricas de tokens con [`scripts/rq1-metrics.mjs`](../../scripts/rq1-metrics.mjs).
4. Revisión ciega: el usuario revisa dos PR en borrador («A» y «B», una ejecución de cada modelo) con la lista de comprobación, y anota sus minutos. Después se revela qué modelo hizo cada uno.
5. Se fusiona el mejor; las demás ramas se conservan sin fusionar.

**Incidencia del primer lanzamiento de T1 (excluida de las métricas):** los *worktrees* de los subagentes se crean desde `main`, no desde la rama de trabajo, así que no contenían la especificación. Las 6 ejecuciones se detuvieron sin hacer nada (≈ 3 llamadas cada una), como pedía el encargo. Al reanudarlas, sus *worktrees* ya se habían borrado (se eliminan solos si no tienen cambios). Se relanzaron 6 ejecuciones nuevas, cuyo encargo empieza con `git merge --ff-only <commit de la especificación>`. El coste de esas 12 transcripciones abortadas se cuenta en el paso, pero no en la comparación entre modelos.

### 4.4 Resultados de T1 (M09 detección y decodificación)

Asignación aleatoria (se reveló tras la revisión): Sonnet 5.5 = r1, r4, r5 · Opus 5.5 = r2, r3, r6. Candidatas de la revisión ciega: **A = r1 (Sonnet)**, **B = r2 (Opus)**.

| Ejecución | Modelo | Llamadas | Lect. caché | Salida estimada (registrada) | USD eq. | Comprobaciones (fallidas) | Pruebas propias | Líneas | Pruebas ocultas |
|---|---|--:|--:|--:|--:|--:|--:|--:|--:|
| r1 (A) | Sonnet 5.5 | 14 | 0,69 M | 23,0 k (0,2 k) | 0,48 | 4 (2) | 25 + 3 doc | 709 | 31/31 |
| r4 | Sonnet 5.5 | 21 | 1,18 M | 30,5 k (0,9 k) | 0,68 | 7 (3) | 30 + 2 doc | 840 | 31/31 |
| r5 | Sonnet 5.5 | 14 | 0,73 M | 24,4 k (2,6 k) | 0,51 | 6 (5) | 25 + 3 doc | 702 | 31/31 |
| r2 (B) | Opus 5.5 | 21 | 1,15 M | 25,0 k (18,0 k) | 0,98 | 6 (4) | 38 + 5 doc | 964 | 31/31 |
| r3 | Opus 5.5 | 19 | 1,07 M | 30,4 k (13,1 k) | 1,12 | 6 (3) | 33 + 5 doc | 923 | 31/31 |
| r6 | Opus 5.5 | 17 | 0,86 M | 24,1 k (20,5 k) | 0,89 | 6 (3) | 46 + 5 doc | 1 119 | 31/31 |

| Modelo | USD eq. por ejecución (media ± sd) | Llamadas (media ± sd) | Comprobaciones fallidas (media) | Revisión humana | Defectos encontrados |
|---|--:|--:|--:|--:|--:|
| **Sonnet 5.5** | **0,56 ± 0,11** | 16,3 ± 4,0 | 3,3 | ≈ 5 min (A) | 0 |
| Opus 5.5 | 1,00 ± 0,11 | 19,0 ± 2,0 | 3,3 | ≈ 5 min (B) | 0 |

**Conclusión para T1:** las 6 ejecuciones superan todos los criterios objetivos, con el mismo número medio de comprobaciones fallidas (en todas, una es el commit rojo de TDD, que falla a propósito) y sin defectos en la revisión. Sonnet 5.5 cuesta un **44 % menos** por ejecución y la diferencia es unas 4 veces la desviación típica, así que es robusta con n = 3. Según la regla de decisión, **gana Sonnet** y su candidata (A) se integra en el proyecto. Opus escribió más pruebas y documentación (+35 % de líneas), pero el revisor no lo percibió como más calidad. En una tarea de especificación cerrada, el *rework* extra que se temía de Sonnet no apareció.

**Validez y limitaciones:**
- **La salida registrada no es fiable en los subagentes.** Algunos mensajes guardan solo el uso parcial del inicio del *streaming*. La salida se estima con el contenido visible (2,1 caracteres por token, calibrado con las sesiones principales, que sí registran el uso completo). El razonamiento interno de esos mensajes no queda registrado, así que los costes son **cotas inferiores**, probablemente más para Opus.
- Es una sola tarea, bien especificada y de dificultad moderada. T2 (FileService, con sistema de ficheros y errores de E/S) dará el segundo punto de datos.
- El coste del orquestador (especificación, pruebas ocultas, evaluación) no depende del modelo evaluado: ≈ 5,1 USD eq. en este paso, que incluye la planificación de F2.

**Coste total del paso 1/8** (hasta abrir el PR #13): ≈ 12,5 USD eq. (228 llamadas), frente a ≈ 7 estimados. Se reparte así:
- orquestador: 88 llamadas, ≈ 6,8 USD (incluye la planificación de F2, la evaluación, la revisión ciega y la documentación);
- 6 ejecuciones válidas: 4,66 USD;
- primer lanzamiento abortado: 34 llamadas, 1,03 USD.

La desviación viene de las incidencias (relanzamiento, investigación del infrarregistro, *scripts* nuevos), que no se repetirán en T2.

### 4.5 Resultados de T2 (M08 FileService y comandos `open_file`/`save_file`)

Sesión B, en el ordenador nuevo (`C:\ProgIA`), con el contexto limpio y sin incidencias: el encargo ya empezaba con `git merge --ff-only c5f2feb`. Asignación aleatoria (revelada tras la revisión): Sonnet 5.5 = r1, r4, r6 · Opus 5.5 = r2, r3, r5. Candidatas sorteadas para la revisión ciega: **A = r1 (Sonnet)**, **B = r3 (Opus)**.

| Ejecución | Modelo | Llamadas | Lect. caché | Salida estimada (registrada) | USD eq. | Comprobaciones (fallidas) | Pruebas propias | Líneas | Pruebas ocultas |
|---|---|--:|--:|--:|--:|--:|--:|--:|--:|
| r1 (A) | Sonnet 5.5 | 17 | 1,11 M | 32,9 k (14,1 k) | 0,77 | 7 (5) | 33 + 3 doc | 900 | 28/28 |
| r4 | Sonnet 5.5 | 20 | 1,29 M | 28,6 k (15,3 k) | 0,76 | 7 (4) | 31 + 2 doc | 884 | 28/28 |
| r6 | Sonnet 5.5 | 20 | 1,17 M | 24,3 k (14,6 k) | 0,68 | 7 (6) | 30 + 2 doc | 872 | 28/28 |
| r2 | Opus 5.5 | 20 | 1,32 M | 34,1 k (9,4 k) | 1,42 | 6 (3) | 42 + 4 doc | 1 240 | 28/28 |
| r3 (B) | Opus 5.5 | 25 | 1,74 M | 31,4 k (15,4 k) | 1,47 | 8 (3) | 44 + 2 doc | 1 165 | 28/28 |
| r5 | Opus 5.5 | 24 | 1,74 M | 28,7 k (15,3 k) | 1,42 | 7 (5) | 37 + 2 doc | 1 024 | 28/28 |

| Modelo | USD eq. por ejecución (media ± sd) | Llamadas (media ± sd) | Comprobaciones fallidas (media) | Revisión humana | Puntuación (1-5: corrección, legibilidad, documentación, pruebas, idiomático) |
|---|--:|--:|--:|--:|---|
| **Sonnet 5.5** | **0,73 ± 0,05** | 19,0 ± 1,7 | 5,0 | 5 min (A) | **5 · 5 · 4 · 5 · 5 (4,8)**, «la fusionaría tal cual» |
| Opus 5.5 | 1,44 ± 0,03 | 23,0 ± 2,6 | 3,7 | 6 min (B) | 4 · 5 · 4 · 4 · 4 (4,2) |

Las 6 ejecuciones superan las 28 pruebas ocultas, clippy, rustfmt y TDD, sin `unwrap`/`expect` fuera de las pruebas. La CI de las dos candidatas pasó en los 4 SO, así que también se probaron sus pruebas solo de Unix (modo del fichero, permiso denegado).

**Conclusión para T2:** gana **Sonnet 5.5** otra vez. Cuesta un **49 % menos** por ejecución, con una diferencia unas 15 veces mayor que la desviación típica, y su candidata recibió **mejor** puntuación con menos minutos de revisión. Opus falló menos comprobaciones intermedias (3,7 frente a 5,0), pero ese *rework* extra de Sonnet es barato: lo detectan las herramientas y lo corrige el propio agente antes de entregar. Opus vuelve a escribir más (+29 % de líneas y más pruebas), y la revisión humana no lo percibe como más calidad. Se integra A (PR #16); B se cierra sin fusionar (PR #17) y las ramas `rq1/T2/*` se conservan.

### 4.6 Respuesta provisional a la RQ.1

| Tarea | Ahorro de Sonnet por ejecución | Calidad (pruebas ocultas) | Revisión ciega | Ganador |
|---|--:|---|---|---|
| T1 M09 lectura | 44 % | 31/31 los dos modelos | equivalente (≈ 5 min cada una) | Sonnet |
| T2 M08 FileService | 49 % | 28/28 los dos modelos | Sonnet mejor (4,8 frente a 4,2) | Sonnet |

**Con especificación cerrada, pruebas automáticas y CI, el modelo barato (Sonnet 5.5) es más rentable:** cuesta la mitad, comete algo más de *rework*, pero las redes de seguridad lo detectan pronto y no llega a la revisión humana ni a `main`. Opus 5.5 sigue siendo la opción del **orquestador** (planificar, especificar, escribir pruebas ocultas, integrar y documentar), donde el contexto es largo y los errores son más caros.

**Limitaciones:** solo 2 tareas de Rust, de dificultad moderada y bien especificadas; un único revisor; costes de los subagentes como cota inferior (salida infrarregistrada); no se ha medido el caso «especificación abierta», donde el criterio del modelo pesa más. Propuesta: delegar en Sonnet los pasos bien especificados de las próximas fases y seguir midiendo.

**Artefactos de T2:** [`rq1/T2-file-service.md`](rq1/T2-file-service.md), [`rq1/T2-hidden-tests.rs`](rq1/T2-hidden-tests.rs), [`rq1/T2-metrics.json`](rq1/T2-metrics.json), [`rq1/T2-runs.json`](rq1/T2-runs.json). Las transcripciones están en el ordenador nuevo (proyecto `C--ProgIA-editorMD`).

## 5. Norma a partir de F2

En cada paso: **estimación previa** (llamadas y USD eq., por analogía con la tabla de la §3) → **medición real** al fusionar el PR → registro aquí y en la web. Cada fase empieza en una sesión nueva, y las fases largas se parten en varias sesiones (F2: A = pasos 1-3, B = pasos 4-8).

**Modelo de estimación:** coste por llamada ≈ 0,04 USD + 0,20 USD por millón de tokens de contexto (ajustado con F0 y F1).

### 5.1 F2: estimación previa frente a coste real

| Paso | Sesión | Estimación (llamadas · USD eq.) | Real (llamadas · USD eq.) |
|---|---|---|---|
| 1/8 Decodificar 🧪 | A | 25 + 6 ejecuciones · ≈ 7 | 228 en total (88 del orquestador) · ≈ 12,5 |
| 2/8 Codificar + fin de línea | A | 25 · 1,9 | 32 · 3,00 |
| 3/8 Pérdidas y transliteración | A | 20 · 1,7 | 30 · 2,62 (+ cierre de la sesión A) |
| 4/8 FileService 🧪 | B | 20 + 6 ejecuciones · ≈ 7 (≈ 9 al empezar el paso) | 195 en total (69 del orquestador) · ≈ 10,9 (6,5 de las 6 ejecuciones) |
| 5/8 FileController | B | 45 · 3,2 | — |
| 6/8 Diálogo de codificación + barra de estado | B | 40 · 3,6 | — |
| 7/8 Ficheros recientes | B | 20 · 2,1 | — |
| 8/8 Cierre F2 | B | 13 · 1,4 | — |
| **Total** | | **≈ 28 (20-38)** | sesión A (pasos 1-3): ≈ 18,1 frente a 10,6 estimados |

**Sesión A:** 290 llamadas en la transcripción, 41 M tokens procesados y ≈ 18,1 USD eq. (16,8 registrados + 1,3 de salida infrarregistrada de los subagentes). Instantánea en [`usage-session3.json`](usage-session3.json). Las desviaciones vienen de las incidencias del experimento (paso 1), de un contexto que llegó a ≈ 315 k tokens por llamada y de los errores de entorno (secuencias de escape en Bash, formato de un JSON). Por eso el cambio a la sesión B se adelantó un paso.

**Artefactos de T1 conservados en el repositorio** (las transcripciones y los *worktrees* son locales): [`rq1/T1-hidden-tests.rs`](rq1/T1-hidden-tests.rs), [`rq1/T1-metrics.json`](rq1/T1-metrics.json), [`rq1/T1-runs.json`](rq1/T1-runs.json) y [`scripts/rq1-eval.sh`](../../scripts/rq1-eval.sh).
