# Visual Rust Studio — Español (España)
#
# Traducción de la interfaz principal. Los archivos designer.ftl y errors.ftl
# todavía no están traducidos: se muestran en inglés (en-US) hasta que la
# comunidad los aporte.

language-name = Español (España)
app-title = Visual Rust Studio
app-title-project = { $project } — Visual Rust Studio
app-tagline = La sencillez del desarrollo visual, la potencia de Rust.

## Menús

menu-file = Archivo
menu-edit = Editar
menu-view = Ver
menu-project = Proyecto
menu-build = Compilar
menu-debug = Depurar
menu-format = Formato
menu-tools = Herramientas
menu-window = Ventana
menu-help = Ayuda
menu-recent-projects = Proyectos recientes

## Comandos

cmd-new-project = Nuevo proyecto…
cmd-open-project = Abrir proyecto…
cmd-save = Guardar
cmd-save-all = Guardar todo
cmd-close-project = Cerrar proyecto
cmd-exit = Salir
cmd-undo = Deshacer
cmd-redo = Rehacer
cmd-cut = Cortar
cmd-copy = Copiar
cmd-paste = Pegar
cmd-delete = Eliminar
cmd-select-all = Seleccionar todo
cmd-find = Buscar y reemplazar…
cmd-format-document = Dar formato al documento (rustfmt)
cmd-view-explorer = Explorador de proyectos
cmd-view-properties = Propiedades
cmd-view-toolbox = Cuadro de herramientas
cmd-view-output = Salida y errores
cmd-view-code = Ver código
cmd-view-designer = Ver diseñador
cmd-add-form = Agregar ventana…
cmd-generate-code = Generar código
cmd-reload-project = Recargar proyecto
cmd-build = Compilar
cmd-run = Ejecutar
cmd-check = Verificar (cargo check)
cmd-clippy = Analizar (Clippy)
cmd-test = Ejecutar pruebas
cmd-clean = Limpiar
cmd-stop = Detener
cmd-release-mode = Modo Release
cmd-debug-start = Iniciar depuración
cmd-settings = Configuración…
cmd-close-tab = Cerrar pestaña
cmd-close-all-tabs = Cerrar todas las pestañas
cmd-about = Acerca de Visual Rust Studio
cmd-rust-book = El libro de Rust (en línea)
cmd-align-left = Alinear a la izquierda
cmd-align-right = Alinear a la derecha
cmd-align-top = Alinear arriba
cmd-align-bottom = Alinear abajo
cmd-align-center-h = Centrar horizontalmente
cmd-align-center-v = Centrar verticalmente
cmd-same-width = Igualar ancho
cmd-same-height = Igualar alto
cmd-same-size = Igualar tamaño
cmd-distribute-h = Distribuir horizontalmente
cmd-distribute-v = Distribuir verticalmente
cmd-bring-front = Traer al frente
cmd-send-back = Enviar al fondo

## Botones comunes

button-save = Guardar
button-cancel = Cancelar
button-ok = Aceptar
button-close = Cerrar
button-browse = Examinar…
button-create = Crear
button-open = Abrir
button-discard = No guardar
button-trust = Confiar y continuar
button-next = Siguiente
button-previous = Anterior
button-replace = Reemplazar
button-replace-all = Reemplazar todo
button-clear = Borrar

## Paneles

panel-explorer = Explorador de proyectos
panel-properties = Propiedades
panel-toolbox = Cuadro de herramientas
panel-output = Salida
panel-errors = Lista de errores
tab-start = Página de inicio
explorer-forms = Ventanas
explorer-files = Archivos
explorer-no-project = No hay ningún proyecto abierto.
explorer-open-designer = Abrir diseñador
explorer-view-code = Ver código

## Página de inicio

start-welcome = Bienvenido a Visual Rust Studio
start-new-project = Crear un proyecto nuevo
start-open-project = Abrir un proyecto
start-recent = Proyectos recientes
start-no-recent = No hay proyectos recientes.
start-learn = Cada ventana que diseña se convierte en código Rust legible que compila con Cargo, con o sin Visual Rust Studio.

## Asistente de nuevo proyecto

newproj-title = Nuevo proyecto
newproj-template = Plantilla
template-gui = Aplicación gráfica Rust
template-gui-desc = Una ventana (Form1) diseñada visualmente, con código Rust generado (egui/eframe).
template-console = Aplicación de consola Rust
template-console-desc = Un programa de línea de comandos.
template-library = Biblioteca Rust
template-library-desc = Una crate reutilizable con una prueba unitaria.
template-existing = Proyecto Cargo existente
template-existing-desc = Abrir una carpeta que ya contiene un Cargo.toml.
newproj-name = Nombre del proyecto
newproj-location = Ubicación
newproj-path-preview = El proyecto se creará en: { $path }
newproj-invalid-name = Use letras, cifras, «-» o «_», empezando por una letra.
newproj-exists = Esta carpeta ya existe.
newproj-choose-folder = Elegir una carpeta

## Agregar ventana

addform-title = Agregar ventana
addform-name = Nombre de la ventana
addform-invalid = El nombre debe empezar por una mayúscula y contener solo letras y cifras (p. ej.: Form2).
addform-exists = Ya existe una ventana con este nombre.

## Configuración

settings-title = Configuración
settings-language = Idioma
settings-language-auto = Automático (sistema: { $lang })
settings-language-chain = Orden de respaldo: { $chain }
settings-language-external = Archivos de traducción adicionales: { $path }
settings-missing-keys = { $count ->
    [0] No falta ninguna traducción.
    [one] Falta una traducción (se muestra en inglés).
   *[other] Faltan { $count } traducciones (se muestran en inglés).
}
settings-appearance = Apariencia
settings-theme = Tema
theme-system = Seguir al sistema
theme-light = Claro
theme-dark = Oscuro
settings-ui-scale = Zoom de la interfaz
settings-editor = Editor de código
settings-editor-font-size = Tamaño de fuente
settings-tab-width = Ancho de sangría
settings-format-on-save = Dar formato con rustfmt al guardar
settings-check-on-save = Verificar el código (cargo check) al guardar
settings-lsp = Usar rust-analyzer (diagnósticos en vivo, autocompletado, información emergente, definición con F12, renombrar con F2)
settings-lsp-missing = rust-analyzer no está instalado: rustup component add rust-analyzer
settings-toolchain = Cadena de herramientas Rust
settings-cargo-path = Ruta de cargo
settings-cargo-path-hint = Déjelo vacío para la detección automática.
settings-cargo-detected = Detectado: { $path }
settings-cargo-not-found = No se encuentra Cargo. Instale Rust desde https://rustup.rs
settings-designer = Diseñador
settings-grid-size = Tamaño de la cuadrícula
settings-snap-to-grid = Ajustar a la cuadrícula
settings-show-grid = Mostrar la cuadrícula
settings-config-file = Archivo de configuración: { $path }

## Barra de estado

status-ready = Listo
status-errors = { $count ->
    [one] { $count } error
   *[other] { $count } errores
}
status-warnings = { $count ->
    [one] { $count } advertencia
   *[other] { $count } advertencias
}
status-running = En ejecución: { $command }
status-build-succeeded = Terminado correctamente.
status-build-failed = Error (código { $code }).
status-process-stopped = Proceso detenido.
status-saved = Guardado: { $file }
status-project-opened = Proyecto abierto: { $name }
status-project-created = Proyecto creado: { $name }
status-code-generated = Código generado ({ $count } archivo(s) actualizado(s)).
status-language-changed = Idioma cambiado.
status-background-check = Verificando el código…
status-release = Release
status-debug-build = Debug

## Mensajes y errores

error-title = Error
error-open-project = No se puede abrir el proyecto: { $error }
error-create-project = No se puede crear el proyecto: { $error }
error-save = No se puede guardar: { $error }
error-cargo-not-found = No se encuentra Cargo. Revise Herramientas → Configuración → Cadena de herramientas Rust.
error-codegen = La generación de código falló: { $error }
error-already-running = Ya hay un comando en ejecución. Deténgalo primero.
error-no-project = Abra o cree primero un proyecto.
error-read-file = No se puede leer el archivo: { $error }

trust-title = Proyecto no confiable
trust-message = Este proyecto no fue creado por Visual Rust Studio. Compilarlo puede ejecutar código (scripts build.rs, macros procedimentales, dependencias). Continúe solo si confía en su origen.
    { $path }

unsaved-title = Cambios sin guardar
unsaved-message = Se han modificado algunos archivos. ¿Desea guardarlos antes de cerrar?

about-title = Acerca de Visual Rust Studio
about-version = Versión { $version }
about-description = Un entorno de desarrollo visual libre para Rust.
about-license = Software libre bajo licencia GNU AGPL-3.0. El código de sus aplicaciones le pertenece.
about-trademarks = Microsoft, Visual Studio, Visual Basic, .NET y Windows Forms son marcas de Microsoft Corporation. Rust es una marca de la Rust Foundation. Visual Rust Studio es un proyecto independiente, no afiliado ni respaldado por Microsoft ni por la Rust Foundation.

## Salida y diagnósticos

output-empty = Todavía no hay salida. Use Compilar → Compilar (Ctrl+Mayús+B) o Ejecutar (F5).
errors-empty = No hay diagnósticos. Ejecute Compilar → Verificar para analizar el código.
diag-level-error = Error
diag-level-warning = Advertencia
diag-level-note = Nota
diag-level-help = Ayuda
diag-explanation = Explicación
diag-original = Mensaje original del compilador
diag-open-location = Ir a la ubicación
diag-suggestion = Corrección sugerida: { $replacement }
filter-errors = Errores
filter-warnings = Advertencias

## Editor de código

editor-find-placeholder = Buscar
editor-replace-placeholder = Reemplazar con
editor-matches = { $total ->
    [0] Sin resultados
    [one] 1 resultado
   *[other] { $total } resultados
}
editor-line-col = Línea { $line }, Col { $col }
editor-modified = Modificado
editor-generated-warning = Este archivo lo genera Visual Rust Studio y se sobrescribirá. Escriba su código en { $file }.

## Comentarios insertados en el código generado

codegen-handler-doc = Gestiona el evento { $event } de `{ $control }`.
codegen-todo = PENDIENTE: escriba su código aquí.
codegen-user-file-doc = Código de la ventana { $form }. Este archivo le pertenece: Visual Rust Studio solo añade nuevos gestionadores de eventos y nunca modifica el código existente.

cmd-set-startup-form = Establecer como ventana de inicio
menu-close-strategy = Al cerrar la ventana principal
close-strategy-main = Cerrar la aplicación
close-strategy-last = Ocultarla; cerrar la aplicación con la última ventana

## Debugging / Débogage

cmd-run-without-debugging = Ejecutar sin depurar
cmd-debug-continue = Continuar
cmd-debug-pause = Pausar
cmd-debug-stop = Detener la depuración
cmd-step-over = Paso a paso por procedimientos
cmd-step-into = Paso a paso por instrucciones
cmd-step-out = Salir de la función
cmd-toggle-breakpoint = Alternar punto de interrupción
cmd-clear-breakpoints = Eliminar todos los puntos de interrupción
panel-debug = Depuración
debug-idle = No hay sesión de depuración. Coloque puntos de interrupción haciendo clic en el margen (o F9) y pulse F5.
debug-call-stack = Pila de llamadas
debug-threads = Subprocesos
debug-locals = Variables locales
debug-watch = Inspección
debug-watch-add = Expresión a inspeccionar
debug-running = En ejecución…
debug-paused = En pausa.
debug-panic = Detenido en un pánico de Rust: vea la pila de llamadas.
debug-stopped = Depuración detenida.
debug-started = Depurando con { $adapter }.
debug-exited = El programa terminó (código { $code }).
debug-no-adapter = No se encontró ningún depurador. Instale GDB 14 o posterior, lldb-dap o CodeLLDB, o indique su ruta en Herramientas → Configuración → Cadena de herramientas Rust.
debug-no-binary = No se encuentra el programa a depurar.
settings-debugger-path = Ruta del depurador
settings-debugger-detected = Depurador: { $adapter }
cmd-add-resource = Agregar una imagen a los recursos…
cmd-rename-form = Cambiar el nombre de la ventana…
rename-form-hint = Se cambian los archivos y el tipo Rust; su código se actualiza (se conserva una copia .bak).
codegen-data-doc = Variables de la ventana (como las variables declaradas en un formulario de VB.NET). Añada sus campos aquí y úselos con `self.data`.
