use super::{QueryUnwrap, VideogameId};
use cynic::GraphQlResponse;
use schema::schema;

// Variables

#[derive(cynic::QueryVariables, Debug, Copy, Clone)]
pub struct VideogameSearchVars<'a> {
    pub name: &'a str,
}

// Query

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "VideogameSearchVars")]
pub struct VideogameSearch {
    #[arguments(query: { filter: { name: $name }, page: 1, perPage: 8 })]
    videogames: Option<VideogameConnection>,
}

#[derive(cynic::QueryFragment, Debug)]
struct VideogameConnection {
    #[cynic(flatten)]
    nodes: Vec<Videogame>,
}

#[derive(cynic::QueryFragment, Debug)]
struct Videogame {
    id: Option<VideogameId>,
    name: Option<String>,
    slug: Option<String>,
}

// Unwrapping

#[derive(Debug, Clone)]
pub struct VideogameData {
    pub id: VideogameId,
    pub name: String,
    pub slug: String,
}

impl<'a> QueryUnwrap<VideogameSearchVars<'a>> for VideogameSearch {
    type Unwrapped = Vec<VideogameData>;

    fn unwrap_response(response: GraphQlResponse<VideogameSearch>) -> Option<Vec<VideogameData>> {
        Some(
            response
                .data?
                .videogames?
                .nodes
                .into_iter()
                .filter_map(|game| {
                    Some(VideogameData {
                        id: game.id?,
                        name: game.name?,
                        slug: game.slug?,
                    })
                })
                .collect(),
        )
    }
}
