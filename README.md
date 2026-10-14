# Doppely
Ассиметричная многопользовательская игра, в которой команда игроков в роли научных сотрудников станции противостоит ИИ-доппельгангеру.

## Компиляция и запуск игры
### Для запуска сервера:
```ps
cargo run --release --bin server --features server
```

### Опционально, для обучения нейросети на видеокарте с CUDA-ядрами можно добавить флаг:
```ps
cargo run --release --bin server --features server cuda
```

### Для запуска клиента:
```ps
cargo run --release --bin client --features client
```
