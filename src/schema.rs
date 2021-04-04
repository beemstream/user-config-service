table! {
    favourite_streams (id) {
        id -> Int4,
        associated_user -> Int4,
        identifier -> Varchar,
        source -> Varchar,
    }
}

table! {
    stream_tag (id) {
        id -> Int4,
        associated_title -> Int4,
        source_id -> Varchar,
        name -> Varchar,
    }
}

table! {
    stream_title (id) {
        id -> Int4,
        associated_user -> Varchar,
        title -> Varchar,
    }
}

joinable!(stream_tag -> stream_title (associated_title));

allow_tables_to_appear_in_same_query!(favourite_streams, stream_tag, stream_title,);
