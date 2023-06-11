// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Binary,
        name -> Text,
        email -> Text,
        password_hash -> Text,
    }
}
