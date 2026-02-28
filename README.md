# Fork Join

## Ejecución

```bash
cargo run <input-path> <num-threads> <output-file-name>
```

por ejemplo

```bash
cargo run ~/Downloads/dataset/deaths 4 output.json
```

## Pruebas

La salida de la ejecución con el dataset completo debe ser igual a la del archivo `expected_output.json`, sin importar el orden de aparición de las keys en los mapas.