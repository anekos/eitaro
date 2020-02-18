table! {
    aliases (id) {
        id -> Integer,
        source -> Text,
        target -> Text,
    }
}

table! {
    definitions (id) {
        id -> Integer,
        term -> Text,
        definition -> Text,
    }
}

table! {
    lemmatizations (id) {
        id -> Integer,
        source -> Text,
        target -> Text,
    }
}

table! {
    levels (id) {
        id -> Integer,
        term -> Text,
        level -> Integer,
    }
}

table! {
    tags (id) {
        id -> Integer,
        term -> Text,
        tag -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    aliases,
    definitions,
    lemmatizations,
    levels,
    tags,
);
