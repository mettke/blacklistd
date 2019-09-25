table! {
    blacklist (ip, ip_type) {
        ip -> Binary,
        ip_type -> Int2,
        backend_type -> Int2,
        last_update -> Timestamp,
    }
}
