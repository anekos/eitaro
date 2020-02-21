
pub mod model;
pub mod schema;



macro_rules! diesel_query {
    ([] $body:expr) => {
        $body
    };

    ([B $($use:ident)*] $body:expr) => {
        {
            use diesel::BoolExpressionMethods;
            diesel_query!([$($use)*] $body)
        }
    };

    ([E $($use:ident)*] $body:expr) => {
        {
            use diesel::ExpressionMethods;
            diesel_query!([$($use)*] $body)
        }
    };

    ([O $($use:ident)*] $body:expr) => {
        {
            use diesel::OptionalExtension;
            diesel_query!([$($use)*] $body)
        }
    };

    ([Q $($use:ident)*] $body:expr) => {
        {
            use diesel::QueryDsl;
            diesel_query!([$($use)*] $body)
        }
    };

    ([R $($use:ident)*] $body:expr) => {
        {
            use diesel::RunQueryDsl;
            diesel_query!([$($use)*] $body)
        }
    };

    ([T $($use:ident)*] $body:expr) => {
        {
            use diesel::TextExpressionMethods;
            diesel_query!([$($use)*] $body)
        }
    };

    ($a:ident [$($use:ident)*] $body:expr) => {
        {
            use crate::db::schema::$a::{dsl as d};
            diesel_query!([$($use)*] $body)
        }
    };

    ($a:ident, $b:ident [$($use:ident)*] $body:expr) => {
        {
            use crate::db::schema::$a::{dsl as d};
            use crate::db::model::$b;
            diesel_query!([$($use)*] $body)
        }
    }
}

