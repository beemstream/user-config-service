table! {
    favourite_streams (id) {
        id -> Int4,
        associated_user -> Int4,
        identifier -> Varchar,
        source -> Varchar,
    }
}
