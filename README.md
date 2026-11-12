# CRIPTA

Roguelite de mazmorra profunda con pixel art procedural, siete clases y progresión permanente. Cada piso es una cámara de una necrópolis astral suspendida en el vacío: 9 a 15 salas conectadas por puertas de piedra que se cierran de golpe al empezar el combate, cofres que guardan llaves, una bóveda sellada, tienda con mercader y deudas, altares de pacto y cientos de criaturas. Juego de escritorio nativo, 100% offline y portable: no usa navegador, Electron ni archivos externos.

![Estado](https://img.shields.io/badge/estado-completo-3fb950) ![Plataformas](https://img.shields.io/badge/Windows%20%7C%20Linux-nativo-58a6ff) ![Sin dependencias](https://img.shields.io/badge/offline-100%25-8b5cf6)

**Código:** [github.com/Guigarmon3/cripta](https://github.com/Guigarmon3/cripta) · **Descarga:** [Releases](https://github.com/Guigarmon3/cripta/releases/latest)

## Jugar

### Windows

1. Descarga `CRIPTA-windows-x64.zip` desde [Releases](https://github.com/Guigarmon3/cripta/releases/latest) y descomprímelo.
2. Abre `cripta.exe`. No hace falta instalar nada. El progreso se guarda en `cripta_save.txt`, junto al ejecutable.

Si SmartScreen muestra un aviso por ser un ejecutable sin firma: **Más información > Ejecutar de todas formas**.

### Linux

Compila en la máquina destino:

```bash
sudo apt install build-essential pkg-config libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev
cargo build --release
./target/release/cripta
```

El binario queda en `target/release/cripta` y también guarda `cripta_save.txt` a su lado.

## Controles

| Acción | Tecla |
|---|---|
| Mover | `WASD` o flechas (reasignable) |
| Atacar | Automático al acercarse al objetivo |
| Esquivar (invulnerable) | `Espacio` (reasignable) |
| Abrir cofre, aceptar pacto o pagar deuda | `E` (reasignable) |
| Abrir el mapa | `Tab` (reasignable) |
| Descender por el portal | Automático al acercarse |
| Curarse en el portal (30 de oro) | `Q` |
| Comprar en la tienda | `1`/`2`/`3`/`4` o clic |
| Comprar a crédito | `F` o clic derecho |
| Pausa | `Esc` (reasignable) |
| Elegir clase | `W`/`S` o `A`/`D`, o clic |
| Jugar | `Enter` |
| Elegir mejora / comprar | `1`/`2`/`3` o clic |
| Cambiar opciones de mejora | `R` (una vez por nivel) |
| Sustituir un ataque equipado | `1`–`6` o clic |
| Abrir el inventario en la pausa | `I` |
| Abrir los contratos de aliado en la taquilla | `T` |
| Abrir la forja desde el menú o la taquilla | `F` |
| Saltar la transición de piso | `Espacio`, `Enter` o clic |
| Abrir ajustes desde la pausa | `O` |
| Abandonar la partida desde la pausa | `Q` |
| Reintentar al morir | `R` |
| Cambiar el nombre en la taquilla | `N` |
| Activar o silenciar la música | `M` |
| Pantalla completa | `F11` |

Todas las acciones de los menús también funcionan con el cursor. La pantalla **Audio** se abre desde el menú principal o desde la pausa. **Ajustes** permite reasignar arriba, abajo, izquierda, derecha, esquiva, mapa, pausa e interactuar; las teclas duplicadas se rechazan con un aviso y el cambio se guarda al instante. La **Biblioteca** reúne ataques, guardianes, criaturas y objetos. La **pausa** concentra continuar, estadísticas, inventario, ajustes y teclas, audio y abandonar; el inventario muestra los seis ataques equipados con su nivel y todos los objetos de la partida.

## Contenido

- **Tres dificultades**: Fácil (el balance original), Medio y Difícil, elegibles en Ajustes y guardadas en el progreso. Cambian vida, daño, velocidad, cooldown, oleadas extra y frecuencia de élites; la dificultad elegida aparece en el HUD.
- **7 clases**: Caballero (bloqueo/represalia), Arquero (crítico/perforación), Mago (área/quema), Asesino (ejecución/velocidad), Nigromante (esqueletos que escalan con tu daño), Clérigo (curación, regeneración y protección) y Mercader (oro, suerte y economía; su oro pesa la mitad). Cada clase tiene stats, figura pixel art animada, arma, ataques y mejoras propias.
- **560 ataques equipados**: 7 clases × 8 arquetipos × 10 elementos. Cada personaje empieza con un ataque de su clase y, al subir de nivel, elige entre ataques nuevos, subir uno equipado o una mejora de estadísticas. Llevas como máximo **6 ataques** y todos se lanzan solos, a la vez, con su propio icono, color elemental, daño, recarga y radio. Si el cupo está lleno y eliges un ataque nuevo, se abre un menú para sustituir uno existente viendo el nombre y la descripción de ambos. Los arquetipos van desde proyectiles, órbitas, auras, novas, cadenas, minas, rastros, vórtices, bolas, rayos, meteoros, enjambres y ondas sísmicas hasta pinchos, drenaje y pulsos de curación.
- **Forja y 21 armas**: tres armas por personaje, compradas con almas y bloqueadas por piso. Cada arma aplica un efecto pasivo al empezar la partida (daño, vida, crítico, esqueletos, oro, suerte, regeneración, bloqueo, perforación, cadena, onda sísmica…). La forja se abre con `F` desde el menú o la taquilla, permite cambiar de personaje con `A`/`D` y forjar o equipar con `1`–`3`. El arma equipada se ve en el inventario.
- **Mazmorra generada**: cada piso es un recorrido aleatorio sobre una rejilla de 5×5 con 9–15 salas conectadas por puertas de piedra de dos hojas. Hay entrada, combate, tienda, descanso, cámara del tesoro, bóveda y sala del guardián, repartidas en cinco tamaños (enana, pequeña, mediana, grande y gigante), cada una con decoración propia. Al entrar en una sala con enemigos las hojas se deslizan desde las jambas y sellan el paso con una runa roja; al limpiarla se abren solas. Las puertas bloqueadas muestran un candado dorado y suenan con un chasquido metálico si intentas cruzarlas.
- **Obstáculos con colisión**: pilares, rocas, huesos y balizas astrales bloquean el paso y también detienen proyectiles. Las salas grandes tienen más escondites; las pequeñas, menos.
- **Cofres y llaves**: tres cofres por piso. Se abren con `E` y entregan una llave y oro; no se rompen a golpes. La cámara del tesoro y la puerta del guardián están selladas: cada llave abre una de esas salas. El cofre del tesoro da un objeto, oro y curación.
- **Llaves negras y bóveda**: las llaves negras son mucho más raras (élites al 8%, cofre del tesoro al 15%, cofres de combate al 5%, tienda desde el piso 3 y la propia bóveda) y solo abren la bóveda. En la Carta de la cripta la bóveda cerrada se marca con una llave lavanda de anilla, caña y dientes, nunca con un borrón negro; al abrirla, con un candado abierto. Dentro espera un objeto legendario, oro y curación.
- **Tienda con mercader**: cuatro ofertas por tienda (curación o núcleo de vida máxima, llave normal o negra, objeto y mejora). La lista se despliega al acercarte al mercader; él comenta tu oro, tu vida y lo que queda por vender. No se puede comprar una curación si ya tienes la vida completa; en ese caso la tienda ofrece un objeto que sube la vida máxima.
- **Crédito y deudas**: si no te alcanza, compra a crédito con `F` o clic derecho. El mercader cobra un recargo del 30% y tiene un límite de fiado. Si mueres endeudado, aparece en la entrada de la siguiente partida a cobrarse: paga con `E` o se llevará parte de tu cristal.
- **El oro pesa de verdad**: cae menos oro que antes y el peso empieza ya a las 90 monedas. Con 350 o más, tu velocidad baja hasta un 30% y tu cadencia de ataque hasta un 12%; los espíritus de codicia también acuden al olor. A veces conviene gastar antes de saquear otro cofre. El **Mercader** carga la mitad del peso, así que puede acumular más antes de frenarse.
- **Guardianes mucho más duros**: un jefe por piso, con 100 nombres posibles (10 títulos × 10 elementos) y 12 siluetas pixel art únicas que no se repiten dentro de la misma partida. Tienen 360 de vida base, pegan 18, atacan más rápido y escalan con fuerza por piso; los pisos 10, 20, 30… multiplican vida y daño. Antes de desatar su firma se hinchan, tiemblan y dibujan un anillo de runas pulsante: es la señal para esquivar. Cada piel tiene su patrón (cruces de luz, espirales, telarañas, pisotones, invocaciones o anillos) y alterna ráfaga, carga, órbita, dispersión y teletransporte. Si un guardián queda acorralado entre obstáculos, salta, avisa con una onda de impacto y se recentra para no atascarse. El guardián final hace sonar todas las capas de la música.
- **240 enemigos normales**: 8 arquetipos × 3 linajes × 10 elementos. Cada arquetipo tiene su ataque: melee, proyectil, dispersión, teledirigido, carga, bomba, aura ralentizadora, invocación, teletransporte, órbita o ráfaga. El elemento cambia el tinte, la voz y el efecto: Escarcha y Veneno ralentizan; Brasa, Sangre y Vacío pegan más fuerte. La dificultad es alta: más vida y daño desde el piso 1, escala de 1,22× por piso y cada enemigo lleva una barra de vida pequeña sobre la cabeza. Los rangos de reconocimiento y las áreas de impacto son cortos: hay que comprometerse para golpear.
- **Enemigos que cantan**: cada arquetipo tiene un motivo vocal breve. Al entrar en combate, la música adopta ese motivo y la sala deja de sonar igual que la anterior.
- **Música por capas**: cada piso añade una capa instrumental (drone, arpegio, coro y percusión). El guardián final las hace sonar todas juntas.
- **Pactos de la Cripta**: la seña de identidad del juego. En cada sala de descanso hay dos altares con una pareja aleatoria de pactos. Solo puedes aceptar uno por sala: una ventaja permanente a cambio de una desventaja permanente. Hay cinco pactos (hierro, fuego, codicia, sangre y tiempo), así que cada partida construye una combinación distinta.
- **Subida de nivel**: eliges 1 de 3 opciones entre **33 mejoras** y los ataques de tu clase, con rarezas, límites de acumulación y una mejora de clase garantizada cada 4 niveles. El modal compacto deja las estadísticas a la izquierda, los valores futuros en verde y las cartas a la derecha; cada carta indica rareza, nivel máximo y el efecto exacto. Puedes cambiar las opciones una vez por nivel con `R`.
- **Objetos y ataques pasivos**: 24 tipos con 5 rarezas que escalan con el piso. Doce añaden ataques y defensas automáticas: sigilo orbital, ojo imán, cadena de huesos, nova de cristal, mina ritual, espejo de dolor, impulso astral, **remolino** que atrae enemigos, **bolas** lentas que perforan y ralentizan, **rastro de brasas**, **aura de ceniza** y **pulsos de curación**. Son pasivos que se suman a los ataques equipados, no ocupan sus seis huecos. El daño en área de los proyectiles es solo el 22% del golpe directo: el impacto limpio sigue siendo el rey. Los élites sueltan raros, los shiny épicos y los guardianes legendarios.
- **Cristal de vida agrietado**: la vida ya no son corazones. Es un sigilo de cristal que se agrieta al recibir daño y aumenta tu daño hasta un 25% cuanto más roto está. El HUD muestra las grietas y el bonus activo.
- **HUD de supervivencia**: la experiencia vive en la barra superior; vida, esquiva, llaves, oro y los seis huecos de ataque están abajo, siempre a la vista. Los avisos aparecen abajo a la derecha sin tapar el combate. El minimapa dibuja la llave de la bóveda en lavanda, del mismo tamaño que las salas y sin borrones negros.
- **Presentación y animaciones**: héroes, enemigos normales y guardianes se dibujan en pixel art procedural con contorno oscuro, paleta limitada y lienzos de píxeles enteros. El héroe anima piernas, torso y brazo del arma en cada paso, y al atacar da una estocada con destello y barrido visible; los enemigos tienen ciclos propios de caminata, aleteo, salto o flotación y gestos de ataque distinguibles (mordisco del limo, maza del bruto, alas del murciélago, orbe del cultista, garras del espectro, bomba del bombardero, lanza del centinela y carga del ojo). Los 12 guardianes tienen silueta, corona, capa, arma, tentáculos o mandíbulas propias, con caminata, flotación y ataque animados. Al aparecer, los voladores caen y los que caminan emergen del suelo entre polvo; al morir se encogen y se desvanecen. Los aliados también se lanzan al golpear, y los guardianes avisan su firma con un anillo de runas. Las auras y órbitas equipadas dejan un anillo persistente con pulsos interiores para que se vea que están actuando. Los guardianes se presentan con un cartel épico, figura pixel art ampliada, halo, bandas cinematográficas y un viaje de cámara del héroe al jefe que bloquea el control unos segundos. El cambio de piso usa un fundido breve que se puede saltar con `Espacio`, `Enter` o clic y desactivar desde **Ajustes**; al derrotar a un guardián, la transición lo recuerda.
- **Dificultad legible**: la vida enemiga escala por rangos según tu daño para evitar que los pisos medios se resuelvan de un golpe; los jefes reciben solo el 65% de ese incremento. Los teletransportes avisan con un anillo y un margen de reacción. Los esqueletos del nigromante reparten daño y duración según cuántos haya, y se recalculan al morir uno. Las salas grandes pueden tener pinchos, ruinas laberínticas y obstáculos; la sala del guardián usa suelo ritual y muros propios. Las puertas muestran emblemas de destino (calavera, moneda, llama, rombo, cerradura, espadas o portal) sin texto, y la Carta de la cripta es una constelación orgánica de nodos y líneas curvadas.
- **Props rompibles**: barriles, cajas, urnas y estatuas sueltan oro, experiencia, curación u objetos. Los cofres son aparte: se abren, no se rompen.
- **Enemigos especiales**: élites dorados, shiny cian que huyen y guardianes con patrones de ataque propios. La dificultad sube: más oleadas desde el piso 2, tres oleadas desde el piso 6 y élites cada 4 pisos.
- **Ecos de partidas muertas**: cada muerte deja un eco con tu clase y el piso alcanzado. En la siguiente partida puedes reclamarlo en la entrada para recibir una bendición menor. La pantalla de estadísticas guarda cuántos ecos vagan por la cripta.
- **Audio separado**: música, efectos de disparo/impacto y sonidos de enemigos tienen volumen independiente. Los enemigos usan nueve voces y cada elemento tiene la suya. La pantalla **Audio** guarda los tres volúmenes en el archivo de progreso.
- **Metaprogresión con desafíos**: las almas se conservan entre partidas y compran mejoras permanentes de vida, daño y suerte. Cada nivel exige haber llegado a un piso concreto además de pagar; la tienda muestra el requisito y bloquea la compra con un sonido de cerradura si aún no lo cumples.
- **Tienda de almas**: precios mucho más altos y aspectos con requisitos de piso y de bajas totales. Desbloquea colores de personaje, gorras y máscaras; los objetos bloqueados se marcan como tales.
- **Taquilla**: equipa los aspectos comprados, cambia de clase y edita tu nombre sin que el campo de texto bloquee el resto de acciones. `Enter` o un clic fuera quitan el foco; mientras escribes, los atajos quedan desactivados. Muestra solo cuántos cosméticos tienes comprados y botones directos a la tienda de almas, a la forja (`F`) y a los contratos de aliado (`T`).
- **Contratos de aliado**: cinco opciones (sin contrato, escudero, farolero, cuervo de la suerte y verdugo). El aliado espectral es invulnerable, te acompaña durante toda la partida y aporta daño cuerpo a cuerpo, curación, oro y suerte o saetas a distancia. El contrato se elige en la taquilla y se aplica al empezar una partida nueva.
- **Menú principal renovado**: fondo de cripta con ladrillos, arco ritual, velas, cadenas y niebla; emblema con arco y cerradura, logotipo trabajado, subtítulo, frases y consejos rotatorios, ficha de clase con lore y barras de estadísticas para las siete clases, y siete accesos directos: Tienda, Taquilla, Forja, Biblioteca, Ajustes, Estadísticas y Audio.
- **Biblioteca de la cripta**: consulta todos los ataques, guardianes, criaturas y objetos con sus descripciones, en fichas paginadas.
- **Ajustes**: reasignación de las ocho acciones de juego, con rechazo de teclas duplicadas, restablecer valores por defecto y acceso al audio.
- **Niebla de guerra ritual**: el mapa no revela cuadros de habitaciones. Solo dibuja líneas de conexión que se van trazando al explorar; la bóveda siempre se marca con su llave lavanda o su candado abierto.
- **Modificadores de piso**: niebla, oscuridad y codicia.
- **Música ambiental procedural**, activable con `M`.

## Identidad: la necrópolis astral

CRIPTA no transcurre en sótanos de ladrillo. Cada piso es una necrópolis suspendida en un abismo de polvo estelar: cámaras de piedra que flotan como plataformas a distintas alturas, sellos rúnicos en las esquinas, un círculo ritual en el centro y puertas de piedra de dos hojas que se cierran de golpe cuando empieza el combate. La interfaz abandona los corazones y la rejilla de habitaciones: la vida es un sigilo de cristal agrietado, el minimapa es una brújula de nodos, el mapa completo es la **Carta de la cripta** (una constelación de salas y líneas que se dibujan), los proyectiles aliados son astillas de luz y los enemigos, orbes sellados con un aro elemental. Los pactos aceptados orbitan al personaje como marcas permanentes. Los objetos de vida máxima son sellos y núcleos de cristal, no corazones.

## Decisiones técnicas

### Rust + macroquad, no web

El requisito era un juego de escritorio real, portable y muy ligero. Rust compila a un binario nativo sin runtime; macroquad da ventana, OpenGL, entrada y audio sin cargar un motor pesado. Evita por completo Electron, navegador y servidores locales. El resultado es un `.exe` de pocos MB que arranca al instante y funciona sin conexión.

### Mundo pixel art nítido, interfaz nativa

El mundo lógico mide 480x270 y se dibuja directamente en ese objetivo interno con `FilterMode::Nearest`; al escalar a la ventana cada píxel del juego cae en un bloque exacto, sin mezclas ni bordes borrosos. Héroes, enemigos y guardianes usan lienzos de píxeles enteros (40x42 para el héroe y los enemigos normales, 56x58 para los jefes), contorno de 1 píxel, paletas cortas y animación por pasos. La interfaz se dibuja **después** del escalado, en píxeles reales de la ventana. Sus coordenadas siguen expresándose en el espacio virtual de 480x270, pero se multiplican por la escala, así que el texto se rasteriza al tamaño final y se ve nítido a cualquier resolución.

Para la lectura se usa **Atkinson Hyperlegible** (regular y negrita), una fuente diseñada para maximizar la distinción entre caracteres. Los textos van en minúsculas y con acentos, y el HUD tiene bandas oscuras de contraste para que se lea sobre cualquier suelo.

### Entrada inmediata y frame variable

La lógica se actualiza una vez por fotograma con el tiempo real transcurrido, limitado a 50 ms para evitar saltos tras una pausa del sistema. Así una pulsación se lee en el mismo fotograma en que ocurre, sin acumuladores que retrasen los menús. Partículas (512) y números flotantes (48) usan pools preasignados; enemigos, proyectiles y pickups reservan capacidad desde el inicio.

### Todo procedural (salvo la fuente)

Sprites pixel art, fondo, viñeta, resplandores y efectos se generan al arrancar desde código; los sonidos y la música se sintetizan en memoria como WAV. La música tiene cuatro capas instrumentales y cada arquetipo enemigo aporta un motivo vocal corto; todo se mezcla con los efectos en tres volúmenes independientes antes de llegar al dispositivo. Las dos variantes de Atkinson Hyperlegible van incrustadas en el binario con `include_bytes!`, así que no hace falta instalar fuentes ni llevar archivos sueltos. La licencia SIL OFL está incluida en `assets/OFL.txt`.

### Guardado portable

`cripta_save.txt` se escribe junto al ejecutable. Si esa carpeta no es escribible, se usa el directorio de trabajo. No se toca el registro, AppData ni ninguna ruta del sistema. Guarda almas, mejoras, nombre, aspectos desbloqueados, aspecto equipado, preferencia de música, los tres volúmenes de audio (música, efectos y enemigos), la deuda pendiente, los ecos de partidas muertas, los contratos de aliado desbloqueados y equipados, las armas forjadas y equipadas por clase, la preferencia de transiciones de piso y las teclas reasignadas (`key0`…`key7`).

### Release pensado para hardware modesto

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

Además, en Windows se enlaza la CRT estáticamente (`+crt-static`) para que el `.exe` no dependa de redistribuibles de Visual Studio. En Windows el binario usa subsistema GUI: no abre ninguna consola.

## Estructura

```
src/
  main.rs      bucle principal, render target, blit, entrada y modo captura
  game.rs      mazmorra, puertas, combate, ataques equipados, cofres, pactos, tienda, deudas, ecos, mapas, biblioteca, forja, ajustes, UI y menús
  attacks.rs   catálogo de 560 ataques por clase, arquetipo y elemento, escalado y ataques iniciales
  weapons.rs   catálogo de 21 armas, efectos pasivos y etiquetas de la forja
  ui.rs        fuente nativa, medidas, texto, paneles y formas
  pixel.rs     pixel art procedural: 7 héroes con cosméticos, 8 enemigos y 12 guardianes
  figures.rs   aliados, cofres, props y portal; conserva el render suave auxiliar
  sprites.rs   helpers de fondo, resplandores y sombras
  audio.rs     efectos, música por capas, motivos vocales y mezcla de volúmenes
  buffs.rs     33 mejoras, rarezas, acumulaciones y pity de clase
  save.rs      metaprogresión, deuda, ecos, cosméticos, contratos, armas, teclas, volúmenes y persistencia
  types.rs     clases, rarezas y RNG xorshift
assets/
  AtkinsonHyperlegible-Regular.ttf
  AtkinsonHyperlegible-Bold.ttf
  OFL.txt
```

## Verificación

```bash
cargo test        # 34 pruebas: RNG, mejoras, pity, costes y requisitos,
                  # arranque, items, mazmorra, cofres, llaves, bóveda, deuda,
                  # ecos, pactos, descanso, tienda, portal, jefes, élites, teclas,
                  # catálogo de 560 ataques, ataques iniciales, clases, 21 armas,
                  # guardado de contratos, armas y transiciones, equilibrio de
                  # daño inicial y peso del oro
cargo build       # comprobación de compilación
cargo build --release
```

Las pruebas cubren la lógica pura y el flujo de cada piso. Además, el modo interno `CRIPTA_SHOT` permite renderizar cada pantalla a PNG (`menu`, `play`, `play_cleric`, `play_merchant`, `combat`, `combat_anim`, `combat_anim2`, `aura`, `levelup`, `replace`, `pause`, `dead`, `victory`, `shop`, `locker`, `forge`, `tickets`, `inventory`, `stats`, `map`, `shoproom`, `rest`, `treasure`, `vault`, `doors`, `entry`, `boss`, `boss_idol`, `boss_skin_0`…`boss_skin_11`, `bossintro`, `boss_clear`, `audio`, `library`, `settings`) para revisar la legibilidad sin depender de la entrada del sistema.

## Ajustes rápidos

- Dificultad: `base_stats` y `spawn_enemy_at` en `game.rs` (`hp_scale` y `dmg_scale`); la vida enemiga por rangos se calcula en `player_power_bracket`.
- Ataques equipados: `attacks.rs` (`archetypes`, `stats`, `catalog`), `MAX_EQUIPPED` y el reparto de opciones en `roll_level_choices`.
- Armas y forja: `weapons.rs` (`CLASS_WEAPONS`, costes y requisitos), `apply_weapon` y `forge_action`.
- Pixel art: `pixel.rs` (`Canvas`, paletas, `draw_hero`, `draw_enemy` y `draw_boss`); los tamaños de lienzo y el anclaje de pies están en `CANVAS_*`, `FEET_Y`, `BOSS_*` y `BOSS_FEET`.
- Jefes y élites: `base_stats`, `spawn_enemy_at` y el bloque `signature_t` de `update_enemies`.
- Número de enemigos y oleadas: `spawn_wave` (`count`).
- Curva de niveles: `check_level_up` (`xp_next`).
- Frecuencia de objetos: `drop_loot`, `update_props` y `roll_rarity`.
- Duración y reparto de esqueletos: `skeleton_life`, `skeleton_share` y `skeleton_damage_for`.
- Salas por piso, cofres y puertas: `generate_dungeon`; apertura y cierre en `update_playing` (`door_seal`).
- Peso del oro: `gold_weight`.
- Precios, requisitos y crédito: `upgrade_cost`, `upgrade_requirement`, `cosmetic_requirement`, `generate_shop_offers` y `buy_shop_offer_inner`.
- Pactos disponibles: `Pact::ALL` y `apply_pact`.
- Teclas reasignables: `KEY_TABLE`, `key_label` y `save::KeyBinds`.
- Capas de música y motivos vocales: `audio.rs` (`set_floor` y `set_motif`).

## Descarga y compilación

La versión para Windows se publica en [Releases](https://github.com/Guigarmon3/cripta/releases/latest). El workflow **Compilar** genera artefactos para Windows y Linux en cada cambio de `main` y en cada etiqueta `v*`.

```bash
cargo build --release
```

El binario queda en `target/release/cripta` (`cripta.exe` en Windows).
