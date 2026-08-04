# S Commie (Rust + Axum)

Reescritura completa de la API original (Python/FastAPI) a **Rust + Axum**,
enfocada en bajo consumo de RAM/CPU, código modular y manejo de errores consistente.

## Formato de respuesta

Todas las rutas devuelven el mismo sobre JSON:

```json
// éxito
{ "status": 200, "data": { ... }, "error": false }

// error
{ "status": 404, "data": { "error": "mensaje", "loc": "query", "param_type": "query" }, "error": true }
```

Esto aplica también a:
- Rutas no encontradas (404, vía `fallback`).
- Parámetros de query faltantes/ inválidos (422, vía un extractor `Query` propio en `src/extract.rs`
  que reemplaza al de axum para no romper el formato de respuesta).
- Panics de handlers individuales no tumban el servidor (cada ruta usa `Result<_, ApiError>`).

## Estructura del proyecto

```
src/
  main.rs            - arranque del servidor, middlewares (CORS, logging, límite de body)
  state.rs           - AppState compartido (cliente HTTP, fuentes, imágenes locales, gifs)
  response.rs         - ApiOk / ApiError (el sobre de respuesta)
  extract.rs          - extractor Query personalizado (errores consistentes)
  validate.rs         - helpers de validación (largo, rango, color hex)
  managers/            - carga en memoria de fuentes (fonts.rs), imágenes locales (images.rs) y gifs (gifs.rs)
  imaging/             - helpers reutilizables: io (fetch/encode), text (wrap/medir/dibujar), effects (máscaras, gradientes, colores dominantes), color (hex)
  routes/json/         - /json/binary, /json/8ball, /json/animegifs, /json/calendar
  routes/image/        - las 29 rutas de manipulación de imágenes (una por archivo)
static/
  fonts/               - AQUÍ van tus .ttf/.otf (mismo naming que ya usabas: Familia.ttf o Familia_Estilo.ttf)
  assets/              - AQUÍ van tus imágenes locales (gru.png, santa.png, wallet.png, etc.)
  gifs.json            - colección de gifs para /json/animegifs (placeholder vacío, ver abajo)
```

## Rutas migradas

**JSON** (sin llamadas a APIs externas): `binary`, `8ball`, `animegifs`, `calendar`.

**Imágenes** (las 29 originales): `grayscale`, `invert`, `mirror`, `blur`, `deepfry`, `pixel`,
`circle`, `color`, `badnews`, `discordjs`, `supreme`, `santa`, `facts`, `sonic`, `titan`,
`twoways`, `thisis`, `beautiful`, `communist`, `rainbow`, `simp`, `sus`, `mad`, `delete`,
`whoreallyare`, `ship`, `rankcard`, `walletcard`, `welcomecard`.

**No migradas** (hacían llamadas a APIs externas, fuera del alcance pedido): weather, currency,
translate, npm, github, anime, youtube, define, runcode, country, imagesearch (bing).

## ⚠️ Antes de correrlo: copia tus assets

El repo trae `static/fonts/` y `static/assets/` **vacíos** (solo `.gitkeep`). Copia ahí
exactamente los mismos archivos que ya tenías en tu proyecto Python:

- `static/fonts/`: mismos `.ttf`/`.otf`, mismo naming (`FranklinGothicDemi.ttf`,
  `GGSans_Medium.ttf`, etc.) — el loader replica la misma convención `Familia_Estilo`.
- `static/assets/`: mismos `.png`/`.jpg` (`gru`, `js`, `santa`, `facts`, `sonic`, `titan`,
  `twoways`, `thisis`, `sus`, `mad`, `delete`, `whoreally`, `simp`, `beautiful`, `wallet`,
  `heart_normal`, `heart_fire`, `heart_broken`, `communism`, `gay`).

Si falta una fuente o un asset, la ruta que lo necesita responde `500` con
`{"error": "Font not found"}` o `{"error": "Asset not found"}` — el servidor entero
sigue funcionando normalmente, no se cae.

**`static/gifs.json`**: no tenía el contenido de tu `static/gifs.py` original (no estaba
en los archivos que me pasaste), así que dejé un JSON con las 15 categorías vacías.
Rellénalo así (mayúsculas en las claves):

```json
{ "ANGRY": ["https://...gif", "https://...gif"], "BAKA": [...], ... }
```

## Desarrollo en VSC

1. Instala Rust (si no lo tienes): https://rustup.rs, o vía tu gestor de paquetes.
2. Copia `static/fonts/` y `static/assets/` con tus archivos reales (ver arriba).
3. Copia `.env.example` a `.env` y ajusta si hace falta (por defecto `PORT=3000`).
4. Corre en modo desarrollo:
   ```bash
   cargo run
   ```
   El primer build compila todas las dependencias (puede tardar 1-3 min); los siguientes
   son incrementales y mucho más rápidos.
5. Prueba, por ejemplo:
   ```bash
   curl "http://localhost:3000/json/8ball?text=hola"
   curl "http://localhost:3000/image/grayscale?image=https://picsum.photos/300" --output out.png
   ```
6. Para probar el binario optimizado (el que se usa en producción):
   ```bash
   cargo build --release
   ./target/release/scommie
   ```

No hay documentación Swagger/Redoc como en FastAPI; en su lugar, `GET /` devuelve la
lista de rutas con su descripción, y `GET /help?query=texto` la filtra.

## Producción en Danbothost (Pterodactyl, Rust ya instalado)

Como ya tienes Rust corriendo en el panel con un "hello world" en axum probado, los
pasos son:

1. **Sube el código** al servidor (copia del repo, como planeas).
2. **Copia tus assets reales** a `static/fonts/` y `static/assets/` dentro del volumen
   del servidor (no vienen en el ZIP/repo por tamaño y porque son tuyos).
3. **Variables de entorno** (defínelas como "Variables" del Egg/Nest en Pterodactyl):
   - `PORT` — Pterodactyl normalmente inyecta el puerto asignado en una variable
     (a veces `SERVER_PORT`). Si tu egg usa un nombre distinto a `PORT`, la forma más
     simple es exportarlo como `PORT` en el **Startup Command**, ej:
     `PORT=${SERVER_PORT} ./target/release/scommie
   - `WEBHOOK` (opcional) — URL de webhook de Discord para logging de requests.
   - `RUST_LOG` (opcional) — `info` por defecto.
4. **Comando de build** (puede ir en el "Install Script" del egg, o ejecutarlo manualmente
   una vez por SSH/consola):
   ```bash
   cargo build --release
   ```
5. **Startup Command** del egg:
   ```bash
   ./target/release/scommie
   ```
   (o `PORT=${SERVER_PORT} ./target/release/scommie` si necesitas mapear la variable de puerto
   de Pterodactyl al nombre que usa la app).
6. El binario final queda en `target/release/scommie` — un solo ejecutable, sin necesidad de
   tener Cargo instalado en tiempo de ejecución (solo en build time). El perfil `release`
   en `Cargo.toml` ya está ajustado (`lto`, `codegen-units=1`, `strip`) para minimizar
   tamaño y consumo de CPU/RAM en runtime.
7. Verifica con:
   ```bash
   curl "http://TU_HOST:PUERTO/"
   ```

### Notas de recursos

- Todas las fuentes e imágenes locales se cargan **una sola vez** al arrancar (`AppState::init`)
  y se comparten entre requests vía `Arc` — no hay I/O de disco repetido por request.
- El límite de tamaño de body está capado a 2MB (esta API no recibe uploads de archivos,
  solo URLs de imágenes y parámetros de texto).
- El cliente HTTP (`reqwest`) se reutiliza (pool de conexiones) en vez de crear uno nuevo
  por request.
- El PNG de salida se codifica directamente desde el buffer crudo (`PngEncoder`), sin
  copias intermedias innecesarias.

## Diferencias / mejoras conscientes vs. la versión Python

- El envelope de respuesta cambió `"success": bool` → `"error": bool` (como pediste).
- `calendar` ahora usa por defecto el año/mes **actual** en vez de tener `2023` fijo.
- El renderizado de emoji a color (Pilmoji) no se migró — el texto se dibuja plano.
  Si lo necesitas, dímelo y lo agregamos con un enfoque de overlay de PNGs de emoji.
- Las tarjetas con "outline"/stroke en botones o barras de progreso son una aproximación
  visual cercana al original (Pillow tiene primitivas de trazo que Rust no replica 1:1),
  pero el resultado es equivalente en la práctica.
