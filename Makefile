DOCKER_COMPOSE=docker-compose -f docker/docker-compose.yaml

# Остановить все контейнеры
docker-stop:
	${DOCKER_COMPOSE} stop

# Остановить и удалить все контейнеры
docker-remove:
	${DOCKER_COMPOSE} down --remove-orphans -v

# Запустить и подготовить базу данных
# Решение по созданию базы с предварительной проверкой ее существования заимствованно отсюда: https://stackoverflow.com/a/36591842/7752659
run-db:
	${DOCKER_COMPOSE} up -d tbot_pg
	sleep 2
	(${DOCKER_COMPOSE} exec -T tbot_pg psql -U postgres -tc "SELECT 1 FROM pg_database WHERE datname = 'tbot'" | grep -q 1) || \
	(${DOCKER_COMPOSE} exec -T tbot_pg psql -U postgres -c "CREATE DATABASE tbot")

# Очистить базу и перезапустить контейнеры
reset-db: docker-remove run-db

# Запустить два сервиса (pr и ipm) в рамках сценария сохранения данных
storing:
	gnome-terminal --tab --title="pr" -- cargo run -p pr -- -c ./pr/configs/ -m ./pr/migrations/ -s
	sleep 2
	gnome-terminal --tab --title="ipm" -- cargo run -p ipm -- -c ./ipm/configs/ -r

# Запустить ipm сервис и два клиента потребителя (на поток trade и поток order book), клиенты из примеров: /ipm/examples/
ipm-real:
	gnome-terminal --tab --title="ipm" -- cargo run -p ipm -- -c ./ipm/configs/ -e
	sleep 2
	gnome-terminal --tab --title="get_trade" -- cargo run -p ipm --example get_trade
	gnome-terminal --tab --title="get_order_book" -- cargo run -p ipm --example get_order_book

# Запустить ipm сервис в режиме ws эмуляции и два клиента потребителя
ipm-emulate:
	gnome-terminal --tab --title="ipm" -- cargo run -p ipm -- -c ./ipm/configs/ -e
	sleep 2
	gnome-terminal --tab --title="get_trade" -- cargo run -p ipm --example get_trade
	gnome-terminal --tab --title="get_order_book" -- cargo run -p ipm --example get_order_book
