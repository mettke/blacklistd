# blacklistd

The Blacklist daemon is a blacklist storage for ipv4 and ipv6 addresses. It uses a blacklist provider to populate and update its internal database  making sure that ip addresses are not blocked forever. 

Implemented providers:

* [abuseipdb](https://abuseipdb.com)

Implemented dababase backends:

* [PostgreSQL](https://www.postgresql.org)
* [MySQL](https://www.mysql.com)
* [SQLite](https://www.sqlite.org/)

# Setup

## IP Expiration

Providers like AbuseIPDB limit their blacklist entries to conserve resources. While a list of 10_000 sounds like pretty much, in reality it is pretty limited considering the amount of malicous ips. To further inrease the limit, we use the additional 3_000 free checks using an ip expiration time.

When an IP is added or updated, its internal timestamp is updated too. If the timestamp of one ip falls below a certain duration (default is 2 weeks), a manual check validates whether it is still malicous or not. If it is, the timestamp is updated and the ip will remain for 2 more weeks. If it isn't, it is removed immediately. This way, we can increase the limit from 10_000 to up to 10_000 + (2 * 7) * 3_000 (52_000).

|Expiration time|Storage for up to X IPs|
|---|---|
|7 days|31_000|
|14 days|52_000|
|21 days|73_000|
|28 days|94_000|

Once we run into the 3_000 free checks per day limit, we will see ips not getting updated anymore. To make sure that those ips are removed, a stale duration was implemented (default 4 weeks). IPs which weren't updated for the given time are removed without further checking.

You can configure both durations using the following arguments:

```sh
blacklistd \
    --expiration-days 14 \
    --stale-days 28 \
    <config...>
```

> Make sure that the stale time is greater then the expiration time. Otherwise ips are removed before being rechecked limiting the storage to the original 10_000

## Provider

At least one provider is required but multiple are allowed.

### AbuseIPDB

AbuseIPDB requires an APIv2 Key. You can get one on you [account page](https://www.abuseipdb.com/account/api). Afterwards, start the daemon using:

```sh
blacklistd \
    --api-abuseipdb "<KEY>" \
    <config..>
```

## Database

A database backend is required to store ip addresses. You can choose between the following three:

## postgresql

Setup a postgres database with user and password. Afterwards it is necessary to run the following migration (optional with docker). After updating the application, it is necessary to run the migration again:

```sh
cargo install diesel_cli

diesel migration run \
    --migration-dir='./migrations.postgres' \
    --database-url='postgres://<user>:<password>@<host>:<port>/<db>'
```

Afterwards you can start the server using:

```sh
blacklistd \
    --db-type postgres \
    --db-host <host> --db-port <port> --db-name <db> \
    --db-user <user> --db-pass <password> \
    <config..>
```

## mysql

Setup a mysql database (or similar like mariadb) with user and password. Afterwards it is necessary to run the following migration (optional with docker). After updating the application, it is necessary to run the migration again:

```sh
cargo install diesel_cli

diesel migration run \
    --migration-dir='./migrations.mysql' \
    --database-url='mysql://<user>:<password>@<host>:<port>/<db>'
```

Afterwards you can start the server using:

```sh
blacklistd \
    --db-type mysql \
    --db-host <host> --db-port <port> --db-name <db> \
    --db-user <user> --db-pass <password> \
    <config..>
```

## sqlite

A dababase file is create when running the following migration. It is, however, also possible to manually create the datbase and to apply the migration afterwards. After updating the application, it is necessary to run the migration again (optional with docker):

```sh
cargo install diesel_cli

diesel migration run \
    --migration-dir='./migrations.sqlite' \
    --database-url='<db_path>'
```

Afterwards you can start the server using:

```sh
blacklistd \
    --db-type sqlite \
    --db-path "<db_path>" \
    <config..>
```

## Docker

There is a docker container which automatically migrates a given database. You can set it up using:

```yaml
  bl-pg:
    image: postgres:alpine
    restart: always
    expose:
      - 5432
    environment:
      POSTGRES_DB: blacklistd
      POSTGRES_USER: blacklistd
      POSTGRES_PASSWORD: <pass>
  bld-pg:
    image: toendeavour/blacklistd
    restart: always
    expose:
      - 8080
    environment:
      DB_TYPE: postgres
      DB_USER: blacklistd
      DB_PASS: <pass>
      DB_HOST: bl-pg
      DB_PORT: 5432
      DB_NAME: blacklistd
      API_ABUSEIPDB: <API>

  bl-sql:
    image: mariadb
    restart: always
    expose:
      - 3306
    environment:
      MYSQL_ROOT_PASSWORD: <root-pass>
      MYSQL_DATABASE: blacklistd
      MYSQL_USER: blacklistd
      MYSQL_PASSWORD: <pass>
    networks:
      - net
  bld-sql:
    image: toendeavour/blacklistd
    restart: always
    expose:
      - 8080
    environment:
      DB_TYPE: mysql
      DB_USER: blacklistd
      DB_PASS: <pass>
      DB_HOST: bl-sql
      DB_PORT: 3306
      DB_NAME: blacklistd
      API_ABUSEIPDB: <API>
    networks:
      - net

  bld-sqlite:
    image: toendeavour/blacklistd
    restart: always
    expose:
      - 8080
    environment:
      API_ABUSEIPDB: <API>
    volumes:
      # User must be set to 100:100
      - "./blacklistd/db.sqlite:/data/db.sqlite"
    networks:
      - net
```

Optional Variables:

|Name|Usage|Default|
|---|---|---|
|PORT|Set Port for the daemon to listen to|8080|
|EXPIRATION_TIME|Set ip expiration time|14|
|STALE_TIME|Set ip stale time|28|
|LOG_LEVEL|Logging Level; Trace (0), Debug (1), Info (2), Warn (3), Error (4), None (5)|2|

# Usage

The daemon provides the following API endpoints:

## API / Blacklist

Provides a blacklist containing every ip currently stored in the database

### JSON

|Entpoint|blacklist|
|---|---|
|Method|GET|
|Accept|application/json|

```sh
curl http://<HOST>:<PORT>/api/blacklist \
    -H "Accept: application/json"

# Response:
# ["10.0.0.1", "10.0.0.2", ...]
```

### Plain

|Entpoint|blacklist|
|---|---|
|Url|api/blacklist|
|Method|GET|
|Accept|text/plain|

```sh
curl http://<HOST>:<PORT>/api/blacklist \
    -H "Accept: text/plain"

# Response:
# 10.0.0.1
# 10.0.0.2
# ...
```

## API / Health

Quick health check. Does not check database health.

|Entpoint|health|
|---|---|
|Url|api/health|
|Method|GET|

```sh
curl http://<HOST>:<PORT>/api/health

# Response:
# True
```

## API / System Health

Full health check.

|Entpoint|system_health|
|---|---|
|Url|api/system_health|
|Method|GET|

```sh
curl http://<HOST>:<PORT>/api/system_health

# Response:
# True
```

## Stats / Count

Number of IPs in the database

### JSON

|Entpoint|count|
|---|---|
|Url|stats/count|
|Method|GET|
|Accept|application/json|

```sh
curl http://<HOST>:<PORT>/stats/count \
    -H "Accept: application/json"

# Response:
# {"count":10486}
```

### Plain

|Entpoint|count|
|---|---|
|Url|stats/count|
|Method|GET|
|Accept|text/plain|

```sh
curl http://<HOST>:<PORT>/stats/count \
    -H "Accept: text/plain"

# Response:
# 10486
```

## Stats / Count per day

Number of IPs in the database grouped by the day of its last check

### JSON

|Entpoint|countPerDay|
|---|---|
|Url|stats/countPerDay|
|Method|GET|
|Accept|application/json|

```sh
curl http://<HOST>:<PORT>/stats/countPerDay \
    -H "Accept: application/json"

# Response:
# [{
#   "count":10000,
#   "last_update_start":"2019-09-30T00:00:16.326944",
#   "last_update_end":"2019-09-30T00:01:21.966092"
# },{
#   "count":486,
#   "last_update_start":"2019-09-28T10:44:15.972453",
#   "last_update_end":"2019-09-28T10:44:40.463461"
# }]
```

### Plain

|Entpoint|countPerDay|
|---|---|
|Url|stats/countPerDay|
|Method|GET|
|Accept|text/plain|

```sh
curl http://<HOST>:<PORT>/stats/countPerDay \
    -H "Accept: text/plain"

# Response:
# 10000 2019-09-30 00:00:16.326944 2019-09-30 00:01:21.966092
# 486 2019-09-28 10:44:15.972453 2019-09-28 10:44:40.463461
```

# Build / Install

You can either checkout the repository and build it using:

```sh
cargo build [--release]
```

or you can install it using:

```sh
cargo install --git https://github.com/mettke/blacklistd.git
```
