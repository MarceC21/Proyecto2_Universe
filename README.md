# Diorama de una nave espacial con Ray Tracing

Proyecto de Gráficas por Computadora desarrollado en **Rust con raylib**. Consiste en un diorama de una nave espacial inspirado en la estética de *Interstellar*, que se puede explorar en primera persona.

## Video de demostración

**Espacio para el video del proyecto:**
[Ver video del diorama](https://www.youtube.com/watch?v=yELFgvXxWZ8)

## Descripción de la escena

La nave está construida principalmente con cubos y bloques rectangulares texturizados. En su interior hay una mesa central con una maqueta animada del sistema solar, una cabina con asientos y controles, un laboratorio, una pequeña estación de trabajo y gabinetes de equipo.

También incluye a **TARS**, construido con cuatro columnas rectangulares que se animan mientras recorre distintas áreas de la nave. El recorrido se repite e incluye pausas en sus estaciones.

La escena cuenta con compuertas automáticas que se activan por proximidad y permiten pasar por la esclusa para salir al espacio y observar el exterior de la nave. Un fondo estelar completa el ambiente.

## Características implementadas

- **Ray tracing:** cálculo de intersecciones de los rayos con la geometría de la escena para determinar el color de cada píxel.
- **Materiales y texturas:** superficies metálicas, paneles, pantallas y vidrio con diferentes propiedades.
- **Iluminación y sombras:** componentes ambiental, difusa y especular, junto con rayos de sombra.
- **Reflexión y refracción:** reflejos en materiales y transmisión de luz a través del vidrio.
- **Animaciones:** movimiento orbital de los planetas, recorrido de TARS y apertura y cierre de compuertas.
- **Cámaras:** exploración en primera persona con colisiones y cámara orbital centrada en la mesa del sistema solar.
- **Optimización:** estructura BVH para acelerar las intersecciones con los cubos y opciones para ajustar la resolución del renderizado.

## Controles

| Tecla o acción | Función |
| --- | --- |
| W, A, S, D | Desplazarse en primera persona. |
| Mouse o flechas | Mirar alrededor en primera persona. |
| C | Alternar entre primera persona y cámara orbital. |
| Flechas en modo orbital | Girar alrededor de la mesa. |
| Rueda del mouse o Q / E | Acercar o alejar la cámara orbital. |
| Tab | Liberar o capturar el cursor en primera persona. |
| Acercarse a los controles de las compuertas | Activar su apertura automática. |
| F1 / F2 / F3 | Usar resolución nativa, 75 % o 50 %. |
| F4 | Activar o desactivar la resolución dinámica. |
| F5 | Mostrar u ocultar los datos de rendimiento. |

## Ejecución

Con Rust y Cargo instalados, ejecutar desde la carpeta que contiene `Cargo.toml`:

```bash
cargo run --release
```

Se recomienda usar el modo `release` para mejorar el rendimiento del trazado de rayos. Si la escena se siente lenta, se puede reducir la resolución con F2 o F3.
