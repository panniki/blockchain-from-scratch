//! Now is your chance to get creative. Choose a state machine that interests you and model it here.
//! Get as fancy as you like. The only constraint is that it should be simple enough that you can
//! realistically model it in an hour or two.
//!
//! Here are some ideas:
//! * Board games:
//!   * Chess
//!   * Checkers
//!   * Tic tac toe
//! * Beaurocracies:
//!   * Beauro of Motor Vehicles - maintains driving licenses and vehicle registrations.
//!   * Public Utility Provider - Customers open accounts, consume the utility, pay their bill periodically, maybe utility prices fluctuate
//!   * Land ownership registry
//! * Tokenomics:
//!   * Token Curated Registry
//!   * Prediction Market
//!   * There's a game where there's a prize to be split among players and the prize grows over time. Any player can stop it at any point and take most of the prize for themselves.
//! * Social Systems:
//!   * Social Graph
//!   * Web of Trust
//!   * Reputation System

use super::{StateMachine, User};
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
enum Proposal {
    Prop1,
    Prop2,
    Prop3,
    Prop4,
}

type Tokens = u32;
type Votes = HashMap<User, Tokens>;

#[derive(Clone, Debug, PartialEq)]
struct ProposalState {
    votes_for: Votes,
    votes_against: Votes,
}

#[derive(Clone, Debug, PartialEq)]
struct Tcr {
    balances: HashMap<User, Tokens>,
    proposals: HashMap<Proposal, ProposalState>,
    registry: Vec<Proposal>,
}

#[derive(Debug, PartialEq, Eq)]
enum Transitions {
    SubmitProposal {
        prop: Proposal,
        user: User,
        stake: Tokens,
    },
    VoteFor {
        prop: Proposal,
        user: User,
        stake: Tokens,
    },
    VoteAgainst {
        prop: Proposal,
        user: User,
        stake: Tokens,
    },
    Resolve {
        prop: Proposal,
    },
}

impl StateMachine for Tcr {
    type State = Tcr;
    type Transition = Transitions;

    fn next_state(init: &Self::State, t: &Self::Transition) -> Self::State {
        let mut new_state = init.clone();

        match t {
            Transitions::SubmitProposal { prop, user, stake } => {
                if new_state.proposals.contains_key(prop) || new_state.registry.contains(prop) {
                    return new_state;
                }

                if Self::adjust_balance(&mut new_state, user, stake).is_err() {
                    return new_state;
                }

                let votes_for = HashMap::from([(*user, *stake)]);
                new_state.proposals.insert(
                    *prop,
                    ProposalState {
                        votes_for,
                        votes_against: HashMap::new(),
                    },
                );
            }

            Transitions::VoteFor { prop, user, stake } => {
                // Proposal niether registered nor submitted
                if !new_state.proposals.contains_key(prop) || new_state.registry.contains(prop) {
                    return new_state;
                }

                if let Some(proposal) = new_state.proposals.get_mut(prop) {
                    // check if voted once, and balance is sufficient
                    if proposal.votes_for.contains_key(user)
                        || proposal.votes_against.contains_key(user)
                        || *stake > *new_state.balances.get(user).unwrap_or(&0)
                    {
                        return new_state;
                    }
                    proposal.votes_for.insert(*user, *stake);
                }

                // can safely adjust balance cuz theck above ensures it's sufficient
                let _ = Self::adjust_balance(&mut new_state, user, stake);
            }

            Transitions::VoteAgainst { prop, user, stake } => {
                // Proposal niether registered nor submitted
                if !new_state.proposals.contains_key(prop) || new_state.registry.contains(prop) {
                    return new_state;
                }

                if let Some(proposal) = new_state.proposals.get_mut(prop) {
                    // check if voted once, and balance is sufficient
                    if proposal.votes_for.contains_key(user)
                        || proposal.votes_against.contains_key(user)
                        || *stake > *new_state.balances.get(user).unwrap_or(&0)
                    {
                        return new_state;
                    }
                    proposal.votes_against.insert(*user, *stake);
                }

                // can safely adjust balance cuz theck above ensures it's sufficient
                let _ = Self::adjust_balance(&mut new_state, user, stake);
            }
            Transitions::Resolve { prop } => {
                // Proposal niether registered nor submitted
                if !new_state.proposals.contains_key(prop) || new_state.registry.contains(prop) {
                    return new_state;
                }
                let total_for: u32 = new_state.proposals[prop].votes_for.values().sum();
                let total_against: u32 = new_state.proposals[prop].votes_against.values().sum();

                if total_against == total_for {
                    let mut users = init.proposals[prop].votes_for.clone();
                    users.extend(init.proposals[prop].votes_against.clone());

                    for (user, stake) in users {
                        let _ = Self::top_up_balance(&mut new_state, &user, &stake);
                    }

                    new_state.proposals.remove(prop);
                } else if total_for > total_against {
                    //for wins
                    let users = init.proposals[prop].votes_for.clone();
                    let top_up = total_against.saturating_div(users.len() as u32);

                    for (user, stake) in users {
                        let new_stake = top_up + stake;
                        let _ = Self::top_up_balance(&mut new_state, &user, &new_stake);
                    }
                    new_state.registry.push(*prop);
                    new_state.proposals.remove(prop);
                } else {
                    // against wins
                    let users = init.proposals[prop].votes_against.clone();
                    let top_up = total_for.saturating_div(users.len() as u32);

                    for (user, stake) in users {
                        let new_stake = top_up + stake;
                        let _ = Self::top_up_balance(&mut new_state, &user, &new_stake);
                    }
                    new_state.proposals.remove(prop);
                }
            }
        }

        new_state
    }
}

impl Tcr {
    fn adjust_balance(state: &mut Tcr, user: &User, stake: &Tokens) -> Result<(), &'static str> {
        match state.balances.get(user) {
            Some(bal) if bal >= stake => {
                let new_bal = bal.saturating_sub(*stake);
                state.balances.insert(*user, new_bal);
                Ok(())
            }
            _ => Err("Insufficient balance"),
        }
    }

    fn top_up_balance(state: &mut Tcr, user: &User, stake: &Tokens) -> Result<(), &'static str> {
        if let Some(bal) = state.balances.get(user) {
            state.balances.insert(*user, bal.saturating_add(*stake));
            Ok(())
        } else {
            Err("User not found")
        }
    }
}

// ========== Helpers ==========
fn initial_state() -> Tcr {
    Tcr {
        balances: HashMap::from([(User::Alice, 100), (User::Bob, 100), (User::Charlie, 100)]),
        proposals: HashMap::new(),
        registry: vec![],
    }
}

// ========== SubmitProposal Tests ==========

#[test]
fn submit_proposal_fails_already_in_proposals() {
    let mut start = initial_state();
    start.proposals.insert(
        Proposal::Prop1,
        ProposalState {
            votes_for: HashMap::new(),
            votes_against: HashMap::new(),
        },
    );
    let end = Tcr::next_state(
        &start,
        &Transitions::SubmitProposal {
            prop: Proposal::Prop1,
            user: User::Alice,
            stake: 10,
        },
    );
    assert_eq!(end, start);
}

#[test]
fn submit_proposal_fails_already_in_registery() {
    let mut start = initial_state();
    start.registry.push(Proposal::Prop1);
    let end = Tcr::next_state(
        &start,
        &Transitions::SubmitProposal {
            prop: Proposal::Prop1,
            user: User::Alice,
            stake: 10,
        },
    );
    assert_eq!(end, start);
}

#[test]
fn submit_proposal_fails_insufficient_balance() {
    let mut start = initial_state();
    start.balances.insert(User::Alice, 50);
    let end = Tcr::next_state(
        &start,
        &Transitions::SubmitProposal {
            prop: Proposal::Prop1,
            user: User::Alice,
            stake: 100,
        },
    );
    assert_eq!(end, start);
}

#[test]
fn submit_proposal_succeeds_adds_to_proposals() {
    let start = initial_state();
    let end = Tcr::next_state(
        &start,
        &Transitions::SubmitProposal {
            prop: Proposal::Prop1,
            user: User::Alice,
            stake: 10,
        },
    );
    assert!(end.proposals.contains_key(&Proposal::Prop1));
}

#[test]
fn submit_proposal_succeeds_deducts_and_adds_vote() {
    let start = initial_state();
    let end = Tcr::next_state(
        &start,
        &Transitions::SubmitProposal {
            prop: Proposal::Prop1,
            user: User::Alice,
            stake: 10,
        },
    );
    let expected = Tcr {
        balances: HashMap::from([(User::Alice, 90), (User::Bob, 100), (User::Charlie, 100)]),
        proposals: HashMap::from([(
            Proposal::Prop1,
            ProposalState {
                votes_for: HashMap::from([(User::Alice, 10)]),
                votes_against: HashMap::new(),
            },
        )]),
        registry: vec![],
    };
    assert_eq!(end, expected);
}

// ========== VoteFor Tests ==========

#[test]
fn vote_for_fails_user_already_voted() {
    let start = Tcr {
        balances: HashMap::from([(User::Alice, 90), (User::Bob, 100), (User::Charlie, 100)]),
        proposals: HashMap::from([(
            Proposal::Prop1,
            ProposalState {
                votes_for: HashMap::from([(User::Alice, 10)]),
                votes_against: HashMap::new(),
            },
        )]),
        registry: vec![],
    };
    let end = Tcr::next_state(
        &start,
        &Transitions::VoteFor {
            prop: Proposal::Prop1,
            user: User::Alice,
            stake: 10,
        },
    );
    assert_eq!(end, start);
}

#[test]
fn vote_for_fails_proposal_not_in_proposals() {
    let start = initial_state();
    let end = Tcr::next_state(
        &start,
        &Transitions::VoteFor {
            prop: Proposal::Prop1,
            user: User::Alice,
            stake: 10,
        },
    );
    assert_eq!(end, start);
}

#[test]
fn vote_for_fails_insufficient_balance() {
    let start = Tcr {
        balances: HashMap::from([(User::Alice, 90), (User::Bob, 50), (User::Charlie, 100)]),
        proposals: HashMap::from([(
            Proposal::Prop1,
            ProposalState {
                votes_for: HashMap::from([(User::Alice, 10)]),
                votes_against: HashMap::new(),
            },
        )]),
        registry: vec![],
    };
    let end = Tcr::next_state(
        &start,
        &Transitions::VoteFor {
            prop: Proposal::Prop1,
            user: User::Bob,
            stake: 100,
        },
    );
    assert_eq!(end, start);
}

#[test]
fn vote_for_succeeds() {
    let start = Tcr {
        balances: HashMap::from([(User::Alice, 90), (User::Bob, 100), (User::Charlie, 100)]),
        proposals: HashMap::from([(
            Proposal::Prop1,
            ProposalState {
                votes_for: HashMap::from([(User::Alice, 10)]),
                votes_against: HashMap::new(),
            },
        )]),
        registry: vec![],
    };
    let end = Tcr::next_state(
        &start,
        &Transitions::VoteFor {
            prop: Proposal::Prop1,
            user: User::Bob,
            stake: 20,
        },
    );
    let expected = Tcr {
        balances: HashMap::from([(User::Alice, 90), (User::Bob, 80), (User::Charlie, 100)]),
        proposals: HashMap::from([(
            Proposal::Prop1,
            ProposalState {
                votes_for: HashMap::from([(User::Alice, 10), (User::Bob, 20)]),
                votes_against: HashMap::new(),
            },
        )]),
        registry: vec![],
    };
    assert_eq!(end, expected);
}

// ========== VoteAgainst Tests ==========

#[test]
fn vote_against_fails_user_already_voted_for() {
    let start = Tcr {
        balances: HashMap::from([(User::Alice, 90), (User::Bob, 100), (User::Charlie, 100)]),
        proposals: HashMap::from([(
            Proposal::Prop1,
            ProposalState {
                votes_for: HashMap::from([(User::Alice, 10)]),
                votes_against: HashMap::new(),
            },
        )]),
        registry: vec![],
    };
    let end = Tcr::next_state(
        &start,
        &Transitions::VoteAgainst {
            prop: Proposal::Prop1,
            user: User::Alice,
            stake: 10,
        },
    );
    assert_eq!(end, start);
}

#[test]
fn vote_against_fails_user_already_voted_against() {
    let start = Tcr {
        balances: HashMap::from([(User::Alice, 90), (User::Bob, 90), (User::Charlie, 100)]),
        proposals: HashMap::from([(
            Proposal::Prop1,
            ProposalState {
                votes_for: HashMap::from([(User::Alice, 10)]),
                votes_against: HashMap::from([(User::Bob, 10)]),
            },
        )]),
        registry: vec![],
    };
    let end = Tcr::next_state(
        &start,
        &Transitions::VoteAgainst {
            prop: Proposal::Prop1,
            user: User::Bob,
            stake: 10,
        },
    );
    assert_eq!(end, start);
}

#[test]
fn vote_against_fails_proposal_not_in_proposals() {
    let start = initial_state();
    let end = Tcr::next_state(
        &start,
        &Transitions::VoteAgainst {
            prop: Proposal::Prop1,
            user: User::Alice,
            stake: 10,
        },
    );
    assert_eq!(end, start);
}

#[test]
fn vote_against_fails_insufficient_balance() {
    let start = Tcr {
        balances: HashMap::from([(User::Alice, 90), (User::Bob, 50), (User::Charlie, 100)]),
        proposals: HashMap::from([(
            Proposal::Prop1,
            ProposalState {
                votes_for: HashMap::from([(User::Alice, 10)]),
                votes_against: HashMap::new(),
            },
        )]),
        registry: vec![],
    };
    let end = Tcr::next_state(
        &start,
        &Transitions::VoteAgainst {
            prop: Proposal::Prop1,
            user: User::Bob,
            stake: 100,
        },
    );
    assert_eq!(end, start);
}

#[test]
fn vote_against_succeeds() {
    let start = Tcr {
        balances: HashMap::from([(User::Alice, 90), (User::Bob, 100), (User::Charlie, 100)]),
        proposals: HashMap::from([(
            Proposal::Prop1,
            ProposalState {
                votes_for: HashMap::from([(User::Alice, 10)]),
                votes_against: HashMap::new(),
            },
        )]),
        registry: vec![],
    };
    let end = Tcr::next_state(
        &start,
        &Transitions::VoteAgainst {
            prop: Proposal::Prop1,
            user: User::Bob,
            stake: 20,
        },
    );
    let expected = Tcr {
        balances: HashMap::from([(User::Alice, 90), (User::Bob, 80), (User::Charlie, 100)]),
        proposals: HashMap::from([(
            Proposal::Prop1,
            ProposalState {
                votes_for: HashMap::from([(User::Alice, 10)]),
                votes_against: HashMap::from([(User::Bob, 20)]),
            },
        )]),
        registry: vec![],
    };
    assert_eq!(end, expected);
}

// ========== Resolve Tests ==========

#[test]
fn resolve_fails_proposal_not_in_proposals() {
    let start = initial_state();
    let end = Tcr::next_state(
        &start,
        &Transitions::Resolve {
            prop: Proposal::Prop1,
        },
    );
    assert_eq!(end, start);
}

#[test]
fn resolve_fails_proposal_already_in_registery() {
    let mut start = initial_state();
    start.registry.push(Proposal::Prop1);
    // Manually add proposal to state (simulating it was already resolved)
    start.proposals.insert(
        Proposal::Prop1,
        ProposalState {
            votes_for: HashMap::from([(User::Alice, 10)]),
            votes_against: HashMap::new(),
        },
    );
    start.balances.insert(User::Alice, 90);

    let end = Tcr::next_state(
        &start,
        &Transitions::Resolve {
            prop: Proposal::Prop1,
        },
    );
    assert_eq!(end, start);
}

#[test]
fn resolve_succeeds_votes_equal() {
    let start = initial_state();
    // Alice submits proposal with 50 tokens
    let after_submit = Tcr::next_state(
        &start,
        &Transitions::SubmitProposal {
            prop: Proposal::Prop1,
            user: User::Alice,
            stake: 50,
        },
    );
    // Bob votes against with 50 tokens
    let after_vote = Tcr::next_state(
        &after_submit,
        &Transitions::VoteAgainst {
            prop: Proposal::Prop1,
            user: User::Bob,
            stake: 50,
        },
    );
    // Resolve
    let end = Tcr::next_state(
        &after_vote,
        &Transitions::Resolve {
            prop: Proposal::Prop1,
        },
    );
    let expected = Tcr {
        balances: HashMap::from([(User::Alice, 100), (User::Bob, 100), (User::Charlie, 100)]),
        proposals: HashMap::new(),
        registry: vec![],
    };
    assert_eq!(end, expected);
}

#[test]
fn resolve_succeeds_votes_against() {
    let start = initial_state();
    // Alice submits proposal with 50 tokens, 50
    let after_submit = Tcr::next_state(
        &start,
        &Transitions::SubmitProposal {
            prop: Proposal::Prop1,
            user: User::Alice,
            stake: 50,
        },
    );
    // Bob votes against with 30 tokens, 70
    let after_vote = Tcr::next_state(
        &after_submit,
        &Transitions::VoteAgainst {
            prop: Proposal::Prop1,
            user: User::Bob,
            stake: 30,
        },
    );
    // Resolve
    let end = Tcr::next_state(
        &after_vote,
        &Transitions::Resolve {
            prop: Proposal::Prop1,
        },
    );
    let expected = Tcr {
        balances: HashMap::from([(User::Alice, 130), (User::Bob, 70), (User::Charlie, 100)]),
        proposals: HashMap::new(),
        registry: vec![Proposal::Prop1],
    };
    assert_eq!(end, expected);
}

#[test]
fn resolve_succeeds_only_votes_for() {
    let start = initial_state();
    // Alice submits proposal with 50 tokens
    let after_submit = Tcr::next_state(
        &start,
        &Transitions::SubmitProposal {
            prop: Proposal::Prop1,
            user: User::Alice,
            stake: 50,
        },
    );
    // Resolve
    let end = Tcr::next_state(
        &after_submit,
        &Transitions::Resolve {
            prop: Proposal::Prop1,
        },
    );
    let expected = Tcr {
        balances: HashMap::from([(User::Alice, 100), (User::Bob, 100), (User::Charlie, 100)]),
        proposals: HashMap::new(),
        registry: vec![Proposal::Prop1],
    };
    assert_eq!(end, expected);
}

#[test]
fn resolve_succeeds_for_wins() {
    let start = initial_state();
    // Alice submits proposal with 60 tokens, alice b = 40
    let after_submit = Tcr::next_state(
        &start,
        &Transitions::SubmitProposal {
            prop: Proposal::Prop1,
            user: User::Alice,
            stake: 60,
        },
    );
    // Bob votes against with 40 tokens, bob b = 60
    let after_vote = Tcr::next_state(
        &after_submit,
        &Transitions::VoteAgainst {
            prop: Proposal::Prop1,
            user: User::Bob,
            stake: 40,
        },
    );
    // Resolve
    let end = Tcr::next_state(
        &after_vote,
        &Transitions::Resolve {
            prop: Proposal::Prop1,
        },
    );
    let expected = Tcr {
        balances: HashMap::from([(User::Alice, 140), (User::Bob, 60), (User::Charlie, 100)]),
        proposals: HashMap::new(),
        registry: vec![Proposal::Prop1],
    };
    assert_eq!(end, expected);
}

#[test]
fn resolve_succeeds_against_wins() {
    let start = initial_state();
    // Alice submits proposal with 40 tokens, alice = 60
    let after_submit = Tcr::next_state(
        &start,
        &Transitions::SubmitProposal {
            prop: Proposal::Prop1,
            user: User::Alice,
            stake: 40,
        },
    );
    // Bob votes against with 60 tokens, bob = 40
    let after_vote = Tcr::next_state(
        &after_submit,
        &Transitions::VoteAgainst {
            prop: Proposal::Prop1,
            user: User::Bob,
            stake: 60,
        },
    );
    // Resolve
    let end = Tcr::next_state(
        &after_vote,
        &Transitions::Resolve {
            prop: Proposal::Prop1,
        },
    );
    let expected = Tcr {
        balances: HashMap::from([(User::Alice, 60), (User::Bob, 140), (User::Charlie, 100)]),
        proposals: HashMap::new(),
        registry: vec![],
    };
    assert_eq!(end, expected);
}
