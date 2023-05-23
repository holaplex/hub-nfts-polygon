use async_graphql::{
    extensions::{ApolloTracing, Logger},
    EmptySubscription, Schema,
};

use crate::graphql::{mutations::Mutation, queries::Query};

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

/// Builds the GraphQL Schema, attaching the Database to the context
#[must_use] pub fn build_schema() -> AppSchema {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .extension(ApolloTracing)
        .extension(Logger)
        .enable_federation()
        .finish()
}
