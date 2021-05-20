Это часть проекта по разработке торгового бота под Тинкофф инвестиции (алготрейдинг), первоначальная задумка проекта изложена 
здесь: https://chugreev.ru/wp/?p=363 Более подробно по архитектуре, проектным решениям и используемым библиотекам я еще 
напишу в отдельных статьях. 

Здесь 2 сервиса, которые я выкладываю в публичный доступ:

* ipm - Incoming Price Manager
* pr - Price Repository

Всего в проекте четыре сервиса, оставшиеся два, относящиеся к финансовой части: расчет индикаторов, построение прогнозных 
моделей, логику принятие решений я не планирую выкладывать.

## Токен

**Обратите, пожалуйста, внимание**. Для того чтобы запустить ipm сервис, необходимо задать токен, который вы можете получить в 
своем личном кабинете в Тинкофф инвестициях, подробнее об этом здесь: https://tinkoffcreditsystems.github.io/invest-openapi/auth/

Для пробного запуска вполне достаточно токена от Sandbox площадки. Для development режима токен необходимо задать в
[ipm/configs/development.yaml](ipm/configs/development.yaml) Вместо `<sandbox token>` нужно вписать токен, взятый с сайта tinkoff.ru

## Подготовка рабочего окружения

Я веду разработку под Linux (Ubuntu 20.04), как это работает под Windows не проверял. Скорее всего, тоже будет работать, 
за исключением разве что make команд.

Для запуска pr сервиса требуется база данных PostgreSQL, вы можете развернуть PostgreSQL локально, а можете воспользоваться Docker 
контейнером. Естественно, сам Docker нужно предварительно [установить](https://docs.docker.com/engine/install/ubuntu/). 
В Makefile есть команда, которая подготовит и запустит контейнер, а также создаст базу данных, если ее еще нет. 

```shell
make run-deps
```

Все подробности по подключению (имя базы и пользователя, порт) можно посмотреть в [pr/configs/default.yaml](pr/configs/default.yaml)

## Запуск сервисов 

Если не указывать RUN_ENV, то по умолчанию будет использован development режим. 

Запуск из корня проекта:
```shell
cargo run -p ipm -- -c ./ipm/configs/
cargo run -p pr -- -c ./pr/configs/ -r 2021-05-11 2
cargo run -p pr -- -c ./pr/configs/ -m ./pr/migrations/ -s
```
Первый флаг `-p` отвечает за выбор проекта. Если запускать из директории проектов, параметр с указанием пути к конфигурационным 
файлам можно отпустить, примеры здесь: [ipm/README.md](ipm/README.md) 

### Запуск сервиса в произвольном режиме

```shell
RUN_ENV=Testing cargo run -p ipm -- -c ./ipm/configs/
RUN_ENV=Production cargo run -p ipm -- -c ./ipm/configs/
```

### Запуск нескольких сервисов

Сценарий сохранения данных (накопление исторических данных)
```shell
make storing
```

Есть другие сценарии: `ipm-real` и `ipm-emulate`, подробности можно посмотреть в [Makefile](Makefile)
