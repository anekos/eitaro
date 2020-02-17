
pub mod model;
pub mod schema;



macro_rules! diesel_query {
    ($a:ident, $body:expr) => {
        {
            use crate::db::schema::$a::dsl::*;
            use diesel::ExpressionMethods;
            use diesel::RunQueryDsl;
            $body
        }
    };

    ($a:ident, $b:ident, $body:expr) => {
        {
            use crate::db::schema::$a::dsl::*;
            use crate::db::model::$b;
            use diesel::ExpressionMethods;
            use diesel::QueryDsl;
            use diesel::RunQueryDsl;
            $body
        }
    }
}

