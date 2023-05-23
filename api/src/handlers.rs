use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_poem::{GraphQLRequest, GraphQLResponse};

use poem::{
    handler,
    web::{
        Data, Html,
    }, IntoResponse, Result,
};


use crate::{AppContext, AppState, UserEmail, UserID};

#[handler]
pub fn health() {}

#[handler]
pub fn playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

#[handler]
pub async fn graphql_handler(
    Data(state): Data<&AppState>,
    user_id: UserID,
    user_email: UserEmail,
    req: GraphQLRequest,
) -> Result<GraphQLResponse> {
    let UserID(user_id) = user_id;
    let UserEmail(user_email) = user_email;

    let context = AppContext::new(state.connection.clone(), user_id, user_email);

    Ok(state.schema.execute(req.0.data(context)).await.into())
}
