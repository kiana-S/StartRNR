use super::{EntrantId, PlayerId, QueryUnwrap, Timestamp, VideogameId};
use cynic::GraphQlResponse;
use schema::schema;

// Variables

#[derive(cynic::QueryVariables, Debug)]
pub struct TournamentSetsVars<'a> {
    // HACK: This should really be an optional variable, but there seems to be a
    // server-side bug that completely breaks everything when this isn't passed.
    pub last_query: Timestamp,

    pub game_id: VideogameId,
    pub country: Option<&'a str>,
    pub state: Option<&'a str>,
}

// Query

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "TournamentSetsVars")]
pub struct TournamentSets {
    #[arguments(query: {
        page: 1,
        perPage: 1,
        sortBy: "endAt desc",
        filter: {
            past: true,
            afterDate: $last_query,
            videogameIds: [$game_id],
            countryCode: $country,
            addrState: $state
        }})]
    pub tournaments: Option<TournamentConnection>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "TournamentSetsVars")]
pub struct TournamentConnection {
    #[cynic(flatten)]
    pub nodes: Vec<Tournament>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "TournamentSetsVars")]
pub struct Tournament {
    pub name: Option<String>,
    #[arguments(limit: 99999, filter: { videogameId: [$game_id] })]
    #[cynic(flatten)]
    pub events: Vec<Event>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Event {
    #[arguments(page: 1, perPage: 999)]
    pub sets: Option<SetConnection>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SetConnection {
    #[cynic(flatten)]
    pub nodes: Vec<Set>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Set {
    #[arguments(includeByes: true)]
    #[cynic(flatten)]
    pub slots: Vec<SetSlot>,
    pub winner_id: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SetSlot {
    pub entrant: Option<Entrant>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Entrant {
    pub id: Option<EntrantId>,
    #[cynic(flatten)]
    pub participants: Vec<Participant>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Participant {
    pub player: Option<Player>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Player {
    pub id: Option<PlayerId>,
}

// Unwrap

pub struct TournamentResponse {
    pub name: String,
    pub sets: Vec<SetResponse>,
}

pub struct SetResponse {
    pub teams: Vec<Vec<PlayerId>>,
    pub winner: usize,
}

impl<'a> QueryUnwrap<TournamentSetsVars<'a>> for TournamentSets {
    type VarsUnwrapped = TournamentSetsVars<'a>;
    type Unwrapped = Vec<TournamentResponse>;

    fn wrap_vars(vars: TournamentSetsVars) -> TournamentSetsVars {
        vars
    }

    // This might be the most spaghetti code I've ever written
    fn unwrap_response(
        response: GraphQlResponse<TournamentSets>,
    ) -> Option<Vec<TournamentResponse>> {
        Some(
            response
                .data?
                .tournaments?
                .nodes
                .into_iter()
                .filter_map(|tour| {
                    let sets = tour
                        .events
                        .into_iter()
                        .filter_map(|event| {
                            Some(
                                event
                                    .sets?
                                    .nodes
                                    .into_iter()
                                    .filter_map(|set| {
                                        let winner_id = set.winner_id?;
                                        let winner = set.slots.iter().position(|slot| {
                                            slot.entrant
                                                .as_ref()
                                                .and_then(|x| x.id)
                                                .map(|id| id.0 == winner_id as u64)
                                                .unwrap_or(false)
                                        })?;
                                        let teams = set
                                            .slots
                                            .into_iter()
                                            .map(|slot| {
                                                slot.entrant?
                                                    .participants
                                                    .into_iter()
                                                    .map(|p| p.player?.id)
                                                    .try_collect()
                                            })
                                            .try_collect()?;
                                        Some(SetResponse { teams, winner })
                                    })
                                    .collect::<Vec<_>>(),
                            )
                        })
                        .flatten()
                        .collect();
                    Some(TournamentResponse {
                        name: tour.name?,
                        sets,
                    })
                })
                .collect(),
        )
    }
}
