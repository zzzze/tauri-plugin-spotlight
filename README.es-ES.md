# Tauri Plugin: Spotlight

Un plugin de Tauri que proporciona una funcionalidad de búsqueda similar a Spotlight de macOS para las ventanas de Tauri.

## Descripción general

Spotlight es un plugin de Tauri que proporciona una forma fácil de usar e intuitiva de interactuar con sus aplicaciones de escritorio: la interfaz de búsqueda similar a Spotlight.

Este plugin está actualmente implementado para macOS, pero tiene implementaciones básicas para otras plataformas.

Características:

1. Permite a los usuarios definir atajos de teclado para mostrar y ocultar la ventana
2. Cualquier ventana puede registrarse para implementar las funciones proporcionadas por este plugin
3. La ventana se ocultará automáticamente al perder el foco
4. Compatible con múltiples monitores (actualmente solo disponible en macOS)
5. La ventana siempre aparecerá en primer plano y reactivará la ventana previamente activa al ocultarse (actualmente solo disponible en macOS)

## Instalación

Instale el plugin principal agregando lo siguiente a su archivo Cargo.toml:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-spotlight = { git = "https://github.com/zzzze/tauri-plugin-spotlight" }
```

Puede instalar los enlaces del invitado de JavaScript usando su gestor de paquetes de JavaScript preferido:

```bash
pnpm add tauri-plugin-spotlight-api
# o
npm add tauri-plugin-spotlight-api
# o
yarn add tauri-plugin-spotlight-api
```

## Uso

### Backend

Hay tres formas de configurar el plugin:

1. Registrar el plugin spotlight con Tauri:

`src-tauri/src/main.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_spotlight::init(Some(tauri_plugin_spotlight::PluginConfig {
            windows: Some(vec![
                tauri_plugin_spotlight::WindowConfig {
                    label: String::from("main"),
                    shortcut: String::from("Ctrl+Shift+J"),
                    macos_window_level: Some(20), // Default 24
                },
            ]),
            global_close_shortcut: Some(String::from("Escape")),
        })))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

2. Configurar el plugin en el archivo de configuración de su aplicación Tauri:

`src-tauri/tauri.conf.json`

```json
{
  "plugins": {
    "spotlight": {
      "windows": [{
        "label": "main",
        "shortcut": "Ctrl+Shift+J",
        "macos_window_level": 20
      }],
      "global_close_shortcut": "Escape"
    }
  }
}
```

`src-tauri/src/main.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_spotlight::init(None))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

3. Registrar manualmente los atajos de teclado de la ventana

`src-tauri/src/main.rs`

```rust
use tauri_plugin_spotlight::ManagerExt;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_spotlight::init(Some(tauri_plugin_spotlight::PluginConfig {
            windows: None,
            global_close_shortcut: Some(String::from("Escape")),
        })))
        .setup(|mut app| {
            if let Some(window) = app.get_window("main") {
                app.spotlight().init_spotlight_window(&window, "Ctrl+Shift+J").unwrap();
            }
            app_modifier::apply(&mut app);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running application");
}
```

Los parámetros de configuración escritos en `tauri.conf.json` y `tauri_plugin_spotlight::init`
se fusionarán automáticamente, con `tauri_plugin_spotlight::init` teniendo mayor prioridad.

### Frontend

Use la función `hide` para hacer invisible una ventana de spotlight:

```typescript
import { hide } from 'tauri-plugin-spotlight-api';

void hide();
```

## Aplicación de ejemplo

### Preparación

1. Compilar la API frontal del plugin.

```bash
pnpm i
pnpm build
```

2. Instalar las dependencias de la aplicación de ejemplo.

```bash
cd examples/react-app
pnpm i
```

3. Iniciar la aplicación de ejemplo.

```bash
pnpm tauri dev
```

## Agradecimientos

Este plugin fue inspirado por el proyecto [tauri-macos-spotlight-example](https://github.com/ahkohd/tauri-macos-spotlight-example)
por [ahkohd](https://github.com/ahkohd), y toma prestado gran parte de su base de código. Gracias a [ahkohd](https://github.com/ahkohd) y a los colaboradores
de [tauri-macos-spotlight-example](https://github.com/ahkohd/tauri-macos-spotlight-example) por su arduo trabajo y contribuciones de código abierto.
