#!/usr/bin/env bash
set +e

SOURCE="${BASH_SOURCE[0]}"
while [ -h "$SOURCE" ]; do # resolve $SOURCE until the file is no longer a symlink
  DIR="$( cd -P "$( dirname "$SOURCE" )" && pwd )"
  SOURCE="$(readlink "$SOURCE")"
  [[ $SOURCE != /* ]] && SOURCE="$DIR/$SOURCE" # if $SOURCE was a relative symlink, we need to resolve it relative to the path where the symlink file was located
done
DIR="$( cd -P "$( dirname "$SOURCE" )" && pwd )"
IFS=$'\n'

DB_HOST="${DB_HOST}"
DB_PORT="3306"
DB_USER="blacklistd"
DB_PASS="blacklistd"
DB_NAME="blacklistd"

ABUSEIPDB_API_KEY="<API_KEY>"
ABUSEIPDB_BLACKLIST="${DIR}/blacklist"

if [[ $(find "${ABUSEIPDB_BLACKLIST}" -mtime +1 -print 2> /dev/null) ]] || 
        [ ! -f "${ABUSEIPDB_BLACKLIST}" ]; then
    curl -sG https://api.abuseipdb.com/api/v2/blacklist \
        -H "Key: ${ABUSEIPDB_API_KEY}" \
        -H "Accept: text/plain" > "${ABUSEIPDB_BLACKLIST}"
fi

cat <<EOF | 
create table ips (
  address decimal(39, 0) not null,
  type tinyint not null,
  last_check timestamp not null default now(),
  PRIMARY KEY (address, type)
  INDEX (last_check)
);
EOF
    mysql -h "${DB_HOST}" -P "${DB_PORT}" -u "${DB_USER}" -p"${DB_PASS}" "${DB_NAME}"

while read ip; do
    ipv4=$(python3 -c "import ipaddress; print(isinstance(ipaddress.ip_address(\"${ip}\"), ipaddress.IPv4Address))")
    ipv6=$(python3 -c "import ipaddress; print(isinstance(ipaddress.ip_address(\"${ip}\"), ipaddress.IPv6Address))")
	if [[ "True" == "${ipv4}" ]]; then
        ip_type=0
	elif [[ "True" == "${ipv6}" ]]; then
        ip_type=1
    else
        continue
	fi
    ip_number=$(python3 -c "import ipaddress; print (int(ipaddress.ip_address(\"${ip}\")))")
    echo "Adding ${ip}"
    echo "insert into ips (address, type, last_check) VALUES(${ip_number}, ${ip_type}, now()) on duplicate key update last_check=now();" |
        mysql -h "${DB_HOST}" -P "${DB_PORT}" -u "${DB_USER}" -p"${DB_PASS}" "${DB_NAME}"
done < "${ABUSEIPDB_BLACKLIST}"

for entry in $(echo "select address, type from ips where last_check < date_sub(now(), interval 1 week ) order by last_check desc;" | mysql -N -h "${DB_HOST}" -P "${DB_PORT}" -u "${DB_USER}" -p"${DB_PASS}" "${DB_NAME}"); do
    address=$(echo "${entry}" | awk '{ print $1 }')
    type=$(echo "${entry}" | awk '{ print $2 }')
    if [[ ${type} -eq 0 ]]; then
        ip=$(python3 -c "import ipaddress; print(ipaddress.IPv4Address(${address}))")
    elif [[ ${type} -eq 1 ]]; then
        ip=$(python3 -c "import ipaddress; print(ipaddress.IPv6Address(${address}))")
    else
        continue
    fi
    RESPONSE=$(curl -G https://api.abuseipdb.com/api/v2/check \
        --data-urlencode "ipAddress=${ip}" \
        -H "Key: ${ABUSEIPDB_API_KEY}" \
        -H "Accept: application/json")
    if [ ${?} -eq 0 ]; then
        SCORE=$(echo ${RESPONSE} | jq .data.abuseConfidenceScore)
        if [[ ${SCORE} -eq 100 ]]; then
            echo "Updating ${ip}"
            echo "insert into ips (address, type, last_check) VALUES(${address}, ${type}, now()) on duplicate key update last_check=now();" |
                mysql -h "${DB_HOST}" -P "${DB_PORT}" -u "${DB_USER}" -p"${DB_PASS}" "${DB_NAME}"
        else
            echo "Deleting ${ip} (Score: ${SCORE})"
            echo "delete from ips where address = ${address} and type = ${type}" |
                mysql -h "${DB_HOST}" -P "${DB_PORT}" -u "${DB_USER}" -p"${DB_PASS}" "${DB_NAME}"
        fi
        sleep 0.5
    else
        break
    fi
    exit 0
done

echo "delete from ips where last_check < date_sub(now(), interval 2 week )" |
    mysql -h "${DB_HOST}" -P "${DB_PORT}" -u "${DB_USER}" -p"${DB_PASS}" "${DB_NAME}"
