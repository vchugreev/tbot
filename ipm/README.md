## Incoming Price Manager

Сервис читает данные о сделках и книге заказов по веб-сокету Тинькофф инвестиции и транслирует их через grpc streaming, 
любой другой сервис может подписаться на grpc streaming и читать эти данные. Примеры подключения можно посмотреть 
здесь: [ipm/examples](ipm/examples)

Варианты запуска:
```shell
cargo run
RUN_ENV=Testing cargo run
RUN_ENV=Production cargo run
```

Запуск с указанием директории, в которой находятся конфиг файлы
```shell
cargo run -- -p ./configs/
```

Запуск в режиме эмуляции ws ридера 
```shell
cargo run -- -e
```

Запуск в режиме подключения к репозиторию (и отправке данных в него)
```shell
cargo run -- -r
```
