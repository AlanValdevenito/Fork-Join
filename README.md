# Fork Join

Trabajo practico para la materia [Técnicas de Programación Concurrente I](https://concurrentes-fiuba.github.io/Inicio.html) en el cual se implementa un programa en Rust para procesar datasets de PUBG y obtener estadísticas sobre jugadores y armas, utilizando concurrencia con el modelo **Fork-Join**. 

El objetivo del trabajo práctico es analizar un dataset con dumps de partidas de PUBG (disponible en [Kaggle](https://www.kaggle.com/datasets/skihikingkevin/pubg-match-deaths)) para generar estadísticas sobre:
1. **Top 10 de armas** más usadas para matar (`killed_by`), con el porcentaje de muertes respecto al total y el promedio de distancia entre asesino y víctima.
2. **Top 10 de jugadores** con más muertes (`killer_name`), con la cantidad total de muertes y el top 3 de armas más usadas por el jugador, con porcentaje de uso.

La idea es aprovechar la concurrencia en Rust para procesar múltiples archivos CSV.

Requisitos
1. Rust stable (última versión).
2. Funciona en **Unix / Linux**.
3. No se permiten crates externos salvo `serde_json` para manejo de JSON.
4. Código libre de `warnings` de compilador y `clippy`.
5. Uso de **cargo fmt** y documentación con **cargo doc**.
6. Cada struct/funcción en archivo independiente.

Se puede acceder al [enunciado](https://concurrentes-fiuba.github.io/2024_2C_tp1.html) del trabajo practico.

## Ejecución

Compilar y ejecutar el programa:

```bash
cargo run <input-path> <num-threads> <output-file-name>
```

Donde:
1. `<input-path>`: Directorio con archivos `.csv`.
2. `<num-threads>`: Cantidad de threads para procesamiento concurrente.
3. `<output-file-name>`: Nombre del archivo de salida JSON.

### Ejemplo de ejecución

```bash
cargo run ~/Downloads/dataset/deaths 4 output.json
```

El formato de salida con el dataset completo debe ser igual a la del archivo [expected_output.json](https://github.com/AlanValdevenito/Fork-Join/blob/main/expected_output.json), sin importar el orden de aparición de las keys en los mapas.

## Concurrencia

Se utiliza el modelo Fork-Join:
1. Cada archivo CSV se procesa de manera paralela en threads separados.
2. Luego se combinan los resultados parciales usando las funciones merge_killers y merge_weapons_stats.

<br>

<img width="739" height="532" alt="image" src="https://github.com/user-attachments/assets/0534635e-60d0-402c-a0cb-caa1235e1aa5" />

<br><br/>

La cantidad de threads se puede ajustar por argumento de línea de comando.
